use axum::{
    body::{Body, HttpBody},
    http::{
        header::{self, HeaderName, HeaderValue},
        uri::{Authority, Parts, Scheme},
        Method, Request, Response, StatusCode, Uri,
        Error as HttpError,
    },
};
use camino::Utf8PathBuf;
use cookie::Cookie;
use futures::TryStreamExt;
use hyper_util::client::legacy::{
    connect::HttpConnector,
    Client,
};
use percent_encoding::percent_decode;
use std::{
    borrow::Cow,
    convert::Infallible,
    fs::create_dir_all,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Semaphore,
    time::timeout,
};
use tower::Service;
use tower_http::services::fs::ServeDir;

const DATA: &str = "data";
const SERVICE: &str = "webservice";

#[derive(Debug)]
pub struct CacheStrategy<B> {
    cache_dir: Utf8PathBuf,
    path: Utf8PathBuf,
    serve_dir: ServeDir,
    semaphore_strategy: SemaphoreStrategy<B>,
}

impl<B> IntoFuture for CacheStrategy<B>
where
    B: Send + HttpBody + Unpin + std::fmt::Debug + 'static,
    <B as HttpBody>::Data: Send,
    <B as HttpBody>::Error: std::error::Error + Send + Sync,
{
    type Output = Result<Response<Body>, Infallible>;
    type IntoFuture = Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send + 'static>>;

    fn into_future(mut self) -> Self::IntoFuture {
        Box::pin(async move {
            // call serve_dir and this be the fallback?
            // need to figure out how to best structure this
            // Assumes this is a `GET` request so body could be ignored later.
            let (parts, body) = self.semaphore_strategy.request.into_parts();
            let cache_path = self.cache_dir.join(&self.path);
            #[cfg(feature = "trace")]
            tracing::debug!("cache_path={cache_path}");
            let cache_headers_path = cache_path.parent().map(|p| p.join("headers"));

            if !cache_path.is_file() {
                // Not a file, clone the parts, pop out the FOUND hader, and get the full content
                // for the cache.
                #[cfg(feature = "trace")]
                tracing::debug!("cache_path={cache_path} not file");
                let mut parts = parts.clone();
                parts.headers.remove(header::RANGE);
                self.semaphore_strategy.request = Request::from_parts(parts, body);

                match self.semaphore_strategy.into_future().await {
                    Err(_) => {
                        #[cfg(feature = "trace")]
                        tracing::info!("proxy.call");
                        return Ok(make_503());
                    }
                    Ok(resp) => {
                        // Any status that isn't 200 OK are returned as is.
                        match resp.status() {
                            StatusCode::OK | StatusCode::FOUND => {}
                            _ => {
                                #[cfg(feature = "trace")]
                                tracing::info!("resp.status={}", resp.status());
                                return Ok(resp);
                            }
                        }
                        if cache_path.parent()
                            .map(create_dir_all)
                            .map(Result::ok)
                            .flatten()
                            .is_none()
                        {
                            #[cfg(feature = "trace")]
                            tracing::error!("create_dir_all={:?}", cache_path.parent());
                            return Ok(make_500());
                        };

                        let (parts, body) = resp.into_parts();

                        if let Ok(file) = File::create(&cache_path).await {
                            #[cfg(feature = "trace")]
                            tracing::debug!("cache_path={:?}", cache_path);
                            if body.into_data_stream()
                                .try_fold(file, |mut file, data| async move {
                                    file.write(&data).await
                                        .map_err(axum::Error::new)?;
                                    Ok(file)
                                })
                                .await
                                .is_err()
                            {
                                // Failed to write to the file be considered an internal error for
                                // now.
                                #[cfg(feature = "trace")]
                                tracing::error!("write={}", cache_path);
                                return Ok(make_500());
                            }

                            // The headers writing can be allowed to fail, but the consequences is
                            // that the original headers won't be set when the proxy responds.
                            if let Some(ref path) = cache_headers_path {
                                if let Ok(mut file) = File::create(path).await {
                                    let _ = file.write(parts.status.as_str().as_bytes()).await;
                                    let _ = file.write(b"\n").await;
                                    for header in [
                                        header::CONTENT_LANGUAGE,
                                        header::CONTENT_TYPE,
                                        header::LOCATION,
                                        header::VARY,
                                    ].iter() {
                                        if let Some(value) = parts.headers.get(header) {
                                            if let Ok(value) = value.to_str() {
                                                let _ = file.write(
                                                    format!(
                                                        "{}: {}\n",
                                                        header,
                                                        value,
                                                    )
                                                    .as_ref(),
                                                ).await;
                                            }
                                        }
                                    }
                                }
                            }

                        } else {
                            // Maybe bypass the cache? For now just error.
                            #[cfg(feature = "trace")]
                            tracing::error!("File::create={}", cache_path);
                            return Ok(make_500());
                        }
                    }
                };
            }

            // The body can be empty as it should have been for the GET request.
            let request = Request::from_parts(parts, Body::empty());
            let (mut parts, body) = self.serve_dir.call(request).await
                .expect("serve_dir should be infallible")
                .into_parts();
            if let Some(ref path) = cache_headers_path {
                if let Ok(mut file) = File::open(path).await {
                    // Just read everything to a `String` and process that for brevity.
                    // Failures are ignored as any missing headers are not too critical here.
                    let mut s = String::new();
                    let _ = file.read_to_string(&mut s).await;
                    let mut lines = s.lines();
                    // Assume status code be 200 on a failure just to get back _something_.
                    let status = StatusCode::from_bytes(lines.next().unwrap_or("200").as_bytes())
                        .unwrap_or(StatusCode::OK);
                    match parts.status {
                        // The default status code from `ServeDir` may be overridden.
                        StatusCode::OK => parts.status = status,
                        // Otherwise leave it as is.
                        _ => (),
                    }
                    for line in s.lines() {
                        let mut split = line.splitn(2, ':');
                        let header = split
                            .next()
                            .map(|s| HeaderName::from_bytes(s.as_ref()));
                        let value = split
                            .next()
                            .map(str::trim)
                            .map(HeaderValue::from_str);
                        if let Some(Ok(header)) = header && let Some(Ok(value)) = value {
                            parts.headers.insert(header, value);
                        }
                    }
                }
            }
            Ok(Response::from_parts(parts, Body::new(body)))
        })
    }
}

#[derive(Debug)]
pub struct SemaphoreStrategy<B> {
    proxy: Proxy<B>,
    semaphore: Arc<Semaphore>,
    semaphore_timeout: Duration,
    request: Request<B>,
}

impl<B> IntoFuture for SemaphoreStrategy<B>
where
    B: Send + HttpBody + Unpin + std::fmt::Debug + 'static,
    <B as HttpBody>::Data: Send,
    <B as HttpBody>::Error: std::error::Error + Send + Sync,
{
    type Output = Result<Response<Body>, Infallible>;
    type IntoFuture = Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send + 'static>>;

    fn into_future(mut self) -> Self::IntoFuture {
        Box::pin(async move {
            match timeout(self.semaphore_timeout, self.semaphore.acquire_owned()).await {
                Ok(Ok(_permit)) => {
                    // Semaphore acquired on-time.
                    self.proxy.call(self.request).await
                }
                Ok(Err(_)) => {
                    // Somehow the Semaphore was closed when it shouldn't have.
                    Ok(make_500())
                }
                Err(_) => {
                    // Timeout acquiring the semaphore.
                    Ok(make_504())
                }
            }
        })
    }
}

#[derive(Debug)]
pub struct PassthruStrategy<B> {
    proxy: Proxy<B>,
    request: Request<B>,
}

impl<B> IntoFuture for PassthruStrategy<B>
where
    B: Send + HttpBody + Unpin + std::fmt::Debug + 'static,
    <B as HttpBody>::Data: Send,
    <B as HttpBody>::Error: std::error::Error + Send + Sync,
{
    type Output = Result<Response<Body>, Infallible>;
    type IntoFuture = Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send + 'static>>;

    fn into_future(mut self) -> Self::IntoFuture {
        Box::pin(async move {
            self.proxy.call(self.request).await
        })
    }
}

/// Specify how the request should be processed.
#[derive(Debug)]
pub enum Strategy<B> {
    /// Use the file at this relative path to cache/load the request.
    Cache(CacheStrategy<B>),
    /// Use the file at this relative path to cache/load the request for the webservice request.
    CacheWebService(CacheStrategy<B>),
    /// Acquire the semaphore before passing through the request to the underlying service.
    Semaphore(SemaphoreStrategy<B>),
    /// Pass through the request to the underlying service unhindered.
    Passthru(PassthruStrategy<B>),
    /// Assume the resource is not found.
    NotFound,
    // TODO Figure out if there is a way to provide an invalidation option, e.g. if POST to certain exposure
    // endpoints, a whole slew of cache may be invalidated.
}


impl<B> IntoFuture for Strategy<B>
where
    B: Send + HttpBody + Unpin + std::fmt::Debug + 'static,
    <B as HttpBody>::Data: Send,
    <B as HttpBody>::Error: std::error::Error + Send + Sync,
{
    type Output = Result<Response<Body>, Infallible>;
    type IntoFuture = Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send + 'static>>;

    fn into_future(self) -> Self::IntoFuture {
        match self {
            Self::Cache(strategy) => {
                strategy.into_future()
            }
            Self::CacheWebService(strategy) => {
                strategy.into_future()
            }
            Self::Semaphore(strategy) => {
                strategy.into_future()
            }
            Self::Passthru(strategy) => {
                strategy.into_future()
            }
            Self::NotFound => {
                Box::pin(async {
                    Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from("<html><body><h1>404 Not Found</h1></body></html>"))
                        .expect("statically constructed response shouldn't error")
                    )
                })
            }
        }
    }
}

impl<B> Strategy<B> {
    pub fn new(gateway: Gateway<B>, request: Request<B>) -> Self {
        if let Some(Ok(cookie)) = request
            .headers()
            .get(header::COOKIE)
            .map(HeaderValue::to_str)
        {
            if Cookie::split_parse(cookie).any(|r| r.map(|c| c.name() == "__ac").unwrap_or(false)) {
                // TODO Should follow this up with our own session management.
                return Strategy::Passthru(PassthruStrategy {
                    proxy: gateway.inner.proxy.clone(),
                    request,
                });
            }
        }

        if request.method() == Method::GET {
            if let Some(path) = resolve_path(request.uri().path()) {
                // For the meantime responses from the web service could be cached if they are GET
                if let Some(Ok(accept)) = request
                    .headers()
                    .get(header::ACCEPT)
                    .map(HeaderValue::to_str)
                {
                    if accept == "application/vnd.physiome.pmr2.json.1" {
                        return Strategy::CacheWebService(CacheStrategy {
                            cache_dir: gateway.inner.cache_service.clone(),
                            path,
                            serve_dir: gateway.inner.serve_dir_service.clone(),
                            semaphore_strategy: SemaphoreStrategy {
                                proxy: gateway.inner.proxy.clone(),
                                semaphore: gateway.inner.cache_semaphore.clone(),
                                semaphore_timeout: gateway.inner.cache_semaphore_timeout,
                                request,
                            }
                        })
                    }
                }

                // Assume **all** GET request can be serviced by the cache.
                // TODO Not all workspace endpoints should be cached.
                return Strategy::Cache(CacheStrategy {
                    cache_dir: gateway.inner.cache_data.clone(),
                    path,
                    serve_dir: gateway.inner.serve_dir_data.clone(),
                    semaphore_strategy: SemaphoreStrategy {
                        proxy: gateway.inner.proxy.clone(),
                        semaphore: gateway.inner.cache_semaphore.clone(),
                        semaphore_timeout: gateway.inner.cache_semaphore_timeout,
                        request,
                    }
                });
            } else {
                // Non-unicode paths are treated as 404 Not Found.
                return Strategy::NotFound;
            }
        }

        // Any other requests assumed to require holding the default semaphore.
        Strategy::Semaphore(SemaphoreStrategy {
            proxy: gateway.inner.proxy.clone(),
            semaphore: gateway.inner.passthru_semaphore.clone(),
            semaphore_timeout: gateway.inner.passthru_semaphore_timeout,
            request,
        })
    }
}

/// The gateway builder.
///
/// Use this to build the final gateway.
#[derive(Debug)]
pub struct GatewayBuilder {
    cache_dir: Utf8PathBuf,
    cache_permits: usize,
    cache_timeout: Duration,
    passthru_permits: usize,
    passthru_timeout: Duration,
}

impl Default for GatewayBuilder {
    fn default() -> Self {
        Self {
            cache_dir: ".".into(),
            cache_permits: 3,
            cache_timeout: Duration::from_secs(10),
            passthru_permits: 3,
            passthru_timeout: Duration::from_secs(10),
        }
    }
}

impl GatewayBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Specify the cache location.
    ///
    /// The default value is the current directory.
    pub fn cache_dir(mut self, cache_dir: impl Into<Utf8PathBuf>) -> Self {
        self.cache_dir = cache_dir.into();
        self
    }

    /// Specify the number of permits allowed for the cache strategy.
    ///
    /// This determines the maximum number of concurrent requests that may be made for requests that will
    /// be cached for data that has not yet been cached.
    pub fn cache_permits(mut self, cache_permits: usize) -> Self {
        self.cache_permits = cache_permits;
        self
    }

    /// Specify the timeout in seconds for the cache strategy.
    pub fn cache_timeout(mut self, cache_timeout: u64) -> Self {
        self.cache_timeout = Duration::from_secs(cache_timeout);
        self
    }

    /// Specify the number of permits allowed for the passthru strategy.
    ///
    /// This determines the maximum number of concurrent requests that may be made for requests that will
    /// not be cached.
    pub fn passthru_permits(mut self, passthru_permits: usize) -> Self {
        self.passthru_permits = passthru_permits;
        self
    }

    /// Specify the timeout in seconds for the passthru strategy.
    pub fn passthru_timeout(mut self, passthru_timeout: u64) -> Self {
        self.passthru_timeout = Duration::from_secs(passthru_timeout);
        self
    }

    /// Build a proxy to a PMR2 instance served for the provided base `Uri`.
    ///
    /// This implementation assumes the provided `Uri` has the scheme of `http` and that no query is
    /// provided with that `Uri`, as that will be ignored.
    pub fn build_for<T, B>(self, base: T) -> Result<Gateway<B>, Box<dyn std::error::Error>>
    where
        B: HttpBody + Send + std::fmt::Debug,
        <B as HttpBody>::Data: Send,
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: std::error::Error + std::fmt::Debug + 'static,
    {
        let base: Uri = base.try_into().map_err(Box::new)?;
        let proxy = Proxy::new(base);

        let cache_data = self.cache_dir.join(DATA);
        let cache_service = self.cache_dir.join(SERVICE);
        create_dir_all(&cache_data)?;
        create_dir_all(&cache_service)?;
        let serve_dir_data = ServeDir::new(&cache_data)
            .redirect_to_trailing_slash(false);
        let serve_dir_service = ServeDir::new(&cache_service)
            .redirect_to_trailing_slash(false);

        let passthru_semaphore = Arc::new(Semaphore::new(self.passthru_permits));
        let passthru_semaphore_timeout = self.passthru_timeout;
        let cache_semaphore = Arc::new(Semaphore::new(self.cache_permits));
        let cache_semaphore_timeout = self.cache_timeout;

        let inner = Arc::new(GatewayInner {
            proxy,
            passthru_semaphore,
            passthru_semaphore_timeout,
            cache_data,
            cache_service,
            cache_semaphore,
            cache_semaphore_timeout,
            serve_dir_data,
            serve_dir_service,
        });
        Ok(Gateway { inner })
    }
}

#[derive(Debug)]
struct GatewayInner<B> {
    proxy: Proxy<B>,

    // These are for the direct access that won't be cached.
    passthru_semaphore: Arc<Semaphore>,
    passthru_semaphore_timeout: Duration,

    // These are to ensure access to resources that have yet to be cached will not overload the underlying.
    cache_semaphore: Arc<Semaphore>,
    cache_semaphore_timeout: Duration,
    cache_data: Utf8PathBuf,
    cache_service: Utf8PathBuf,

    // The services for the relevant underlying data.
    serve_dir_data: ServeDir,
    serve_dir_service: ServeDir,
}

#[derive(Debug)]
pub struct Gateway<B> {
    inner: Arc<GatewayInner<B>>,
}

impl<B> Clone for Gateway<B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<ReqBody> Service<Request<ReqBody>> for Gateway<ReqBody>
where
    ReqBody: Send + HttpBody + Unpin + std::fmt::Debug + 'static,
    <ReqBody as HttpBody>::Data: Send,
    <ReqBody as HttpBody>::Error: std::error::Error + Send + Sync,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let service = self.clone();
        let strategy = Strategy::new(service, req);
        #[cfg(feature = "trace")]
        tracing::debug!("strategy={strategy:?}");
        strategy.into_future()
    }
}

#[derive(Debug)]
struct ProxyInner<B> {
    client: Arc<Client<HttpConnector, B>>,
    authority: Authority,
    uri_base: Cow<'static, str>,
}

#[derive(Debug)]
pub struct Proxy<B> {
    inner: Arc<ProxyInner<B>>,
}

impl<B> Clone for Proxy<B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<B> Proxy<B>
where
    B: HttpBody + Send + std::fmt::Debug,
    <B as HttpBody>::Data: Send,
{
    /// Create a proxy to a PMR2 instance served at the provided base `Uri`.
    ///
    /// This implementation assumes the provided `Uri` has the scheme of `http` and that no query is
    /// provided as that will be ignored.
    pub fn new<T>(base: T) -> Self
    where
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: std::fmt::Debug,
    {
        let Parts { authority, path_and_query, .. } = base
            .try_into()
            .expect("provided base uri must be valid")
            .into_parts();
        let authority = authority
            .expect("authority must be provided by the base uri");
        // Assume the service exists for the duration of the program, leaking this for convenience.
        let uri_base = path_and_query
            .expect("path must be provided by the provided base uri")
            .path()
            .trim_end_matches('/')
            .to_owned()
            .into();

        let client = Arc::new(Client::builder(hyper_util::rt::TokioExecutor::new()).build_http());
        let inner = Arc::new(ProxyInner {
            client,
            authority,
            uri_base,
        });
        Self { inner }
    }

    pub fn patch_request(&self, request: &mut Request<B>) -> Result<(), HttpError> {
        let request_uri = request.uri();
        let request_path = request_uri.path();
        let request_query = request_uri.query();
        let mut target = String::with_capacity(self.inner.uri_base.len()
            + request_path.len()
            + request_query
                .map(|q| q.len() + 1)
                .unwrap_or(0)
        );
        target.push_str(&self.inner.uri_base);
        target.push_str(request_path);
        if let Some(request_query) = request_query {
            target.push('?');
            target.push_str(request_query);
        }

        let uri = Uri::builder()
            .scheme(Scheme::HTTP)
            .authority(self.inner.authority.clone())
            .path_and_query(target)
            .build()?;

        *request.uri_mut() = uri;
        Ok(())
    }
}

impl<ReqBody> Service<Request<ReqBody>> for Proxy<ReqBody>
where
    ReqBody: Send + HttpBody + Unpin + std::fmt::Debug + 'static,
    <ReqBody as HttpBody>::Data: Send,
    <ReqBody as HttpBody>::Error: std::error::Error + Send + Sync,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let service = self.clone();
        Box::pin(async move {
            // FIXME An error here should trigger a 503 response and the appropriate log.
            let _ = service.patch_request(&mut req);

            let resp = service.inner.client.as_ref().request(req).await
                .map(|resp| {
                    let (parts, incoming) = resp.into_parts();
                    let body = Body::new(incoming);
                    Response::from_parts(parts, body)
                })
                .unwrap_or_else(|_| make_503());
            Ok(resp)
        })
    }
}

// TODO Build a themed response off the following.
fn make_500() -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from("<html><body><h1>500 Internal Server Error</h1></body></html>"))
        .expect("statically constructed response shouldn't error")
}

fn make_503() -> Response<Body> {
    Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from("<html><body><h1>503 Servce Unavailable</h1></body></html>"))
        .expect("statically constructed response shouldn't error")
}

fn make_504() -> Response<Body> {
    Response::builder()
        .status(StatusCode::GATEWAY_TIMEOUT)
        .header(header::CONTENT_TYPE, "text/html")
        .body(Body::from("<html><body><h1>504 Gateway Timeout</h1></body></html>"))
        .expect("statically constructed response shouldn't error")
}

// This resolves to the filesystem equivalent path.  Given that every resource may have a trailing slash
// and that the resource will have an html presentation, a concrete `index.html` filename will be appended
// to them to match with typical behavior http servers (like `ServeDir`) where it will serve the request
// uri with the trailing slash with that file.  This complicates the issue if a valid uri also contain a
// concrete `index.html` in one of its segment - the resulting segment will have `index.html` as a path
// segment that will be some directory, and that will definitely conflict with the cache file of the same
// name.
//
// The path returned will be some relative path - this is intended as it will be joined with the absolute
// path provided by the service that will consume this value.
//
// Given that no PMR2 resources (currently as of writing) has `index_html` as a directory or a file, this
// function simply converts any segements of `index.html` to `index_html`.  This can conflict with Plone's
// use of this filename but that is restricted to portal content (and view path) which shouldn't cause
// actual issues in the current scenario, but naturally there will be confusion if there was indeed a
// file (or directory) named `index_html` and `index.html` at the same level.
//
// A cleaner workaround would be just to hash all paths into some string, but this way there will be a
// concrete path of the requested resource on the filesystem for direct inspection.  Once this is settled
// using the hash can be done later if the naming conflict (between `index_html` and `index.html`) becomes
// a real issue.
fn resolve_path(path: &str) -> Option<Utf8PathBuf> {
    let mut segments = Vec::new();
    // This is inline with the behavior of `ServeDir` (as setup with `axum`) - it will resolve `..` that
    // traverse above the root remain within the underlying root and will resolve to the correct file without
    // allowing directory traversal beyond the defined root.
    for segment in percent_decode(path.as_ref())
        .decode_utf8()
        .ok()?
        .split("/")
        .map(|s| {
            if s == "index.html" {
                "index_html".to_owned()
            } else {
                format!("{s}")
            }
        })
    {
        if segment == "." {
            continue;
        } else if segment == ".." {
            segments.pop();
        } else {
            segments.push(segment)
        }
    }
    segments.push("index.html".to_owned());
    Some(segments.into_iter().collect::<Utf8PathBuf>())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn proxy_patch_request() {
        let service = Proxy::new("http://example.com/pmr/");
        let mut req = Request::builder()
            .method(Method::POST)
            .uri("/workspace/123")
            .body(Body::empty())
            .expect("valid request");
        service.patch_request(&mut req)
            .expect("request should have been successfully patched");
        assert_eq!(req.uri(), "http://example.com/pmr/workspace/123");
    }

    #[test]
    fn smoke_resolve_path() {
        fn resolve_path_unwrapped(s: &str) -> Utf8PathBuf {
            resolve_path(s).unwrap()
        }

        assert_eq!(resolve_path_unwrapped("/"), "index.html");
        assert_eq!(resolve_path_unwrapped("/test"), "test/index.html");
        assert_eq!(resolve_path_unwrapped("/test/"), "test/index.html");
        assert_eq!(resolve_path_unwrapped("/test/target"), "test/target/index.html");
        assert_eq!(resolve_path_unwrapped("/test%2ftarget"), "test/target/index.html");
        assert_eq!(resolve_path_unwrapped("/test//target"), "test/target/index.html");
        assert_eq!(resolve_path_unwrapped("/test///target"), "test/target/index.html");
        assert_eq!(resolve_path_unwrapped("/index_html"), "index_html/index.html");
        assert_eq!(resolve_path_unwrapped("/index.html"), "index_html/index.html");
        assert_eq!(resolve_path_unwrapped("/test/index_html"), "test/index_html/index.html");
        assert_eq!(resolve_path_unwrapped("/test/index.html"), "test/index_html/index.html");

        // Segments with special names (i.e. curdir/pardir) should be ignored.
        assert_eq!(resolve_path_unwrapped("/../target"), "target/index.html");
        assert_eq!(resolve_path_unwrapped("/../../../../target"), "target/index.html");
        assert_eq!(resolve_path_unwrapped("/dir/../target"), "target/index.html");
        assert_eq!(
            resolve_path_unwrapped("/workspace/a1/file/../file/main"),
            "workspace/a1/file/main/index.html",
        );
        assert_eq!(
            resolve_path_unwrapped("/workspace/a1/file/../file/main/"),
            "workspace/a1/file/main/index.html",
        );

        // Plone usually has the `/view` suffix being the same as the underlying resource, but for now this
        // resolver ignores this issue as the optimization can be done by the underlying service.
        assert_eq!(resolve_path_unwrapped("/welcome"), "welcome/index.html");
        assert_eq!(resolve_path_unwrapped("/welcome/view"), "welcome/view/index.html");

        // Invalid unicode is decoded to nothing
        assert_eq!(resolve_path("/%e7"), None);
    }
}
