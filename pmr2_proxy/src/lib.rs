use axum::{
    body::{Body, HttpBody},
    http::{
        header::HeaderValue,
        uri::{Authority, Parts, Scheme},
        Method, Request, Response, StatusCode, Uri, header,
        Error as HttpError,
    },
};
use camino::Utf8PathBuf;
use cookie::Cookie;
use hyper_util::client::legacy::{
    connect::HttpConnector,
    Client,
};
use std::{
    borrow::Cow,
    convert::Infallible,
    fs::create_dir_all,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::Semaphore;
use tower::Service;
use tower_http::services::fs::ServeDir;

const DATA: &str = "data";
const SERVICE: &str = "webservice";

/// Specify how the request should be processed.
#[derive(Debug)]
pub enum Strategy<B> {
    /// Use the file at this relative path to cache/load the request.
    Cache {
        path: Utf8PathBuf,
        serve_dir: ServeDir,
        proxy: Proxy<B>,
        request: Request<B>,
    },
    /// Use the file at this relative path to cache/load the request for the webservice request.
    CacheWebService {
        path: Utf8PathBuf,
        serve_dir: ServeDir,
        proxy: Proxy<B>,
        request: Request<B>,
    },
    /// Acquire the semaphore before passing through the request to the underlying service.
    Semaphore {
        proxy: Proxy<B>,
        request: Request<B>,
    },
    /// Pass through the request to the underlying service unhindered.
    Passthru {
        proxy: Proxy<B>,
        request: Request<B>,
    },
    /// Assume the resource is not found.
    NotFound,
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
                return Strategy::Passthru {
                    proxy: gateway.inner.proxy.clone(),
                    request,
                };
            }
        }

        if request.method() == Method::GET {
            // For the meantime responses from the web service could be cached if they are GET
            if let Some(Ok(accept)) = request
                .headers()
                .get(header::ACCEPT)
                .map(HeaderValue::to_str)
            {
                if accept == "application/vnd.physiome.pmr2.json.1" {
                    return Strategy::CacheWebService {
                        path: resolve_path(request.uri().path()),
                        serve_dir: gateway.inner.serve_dir_service.clone(),
                        proxy: gateway.inner.proxy.clone(),
                        request,
                    }
                }
            }

            // Assume **all** GET request can be serviced by the cache.
            // TODO Not all workspace endpoints should be cached.
            return Strategy::Cache {
                path: resolve_path(request.uri().path()),
                serve_dir: gateway.inner.serve_dir_data.clone(),
                proxy: gateway.inner.proxy.clone(),
                request,
            };
        }

        // Any other requests assumed to require holding the semaphore.
        Strategy::Semaphore {
            proxy: gateway.inner.proxy.clone(),
            request,
        }
    }
}

#[derive(Debug)]
struct GatewayInner<B> {
    proxy: Proxy<B>,
    semaphore: Semaphore,
    cache_data: Utf8PathBuf,
    cache_service: Utf8PathBuf,
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

impl<B> Gateway<B>
where
    B: HttpBody + Send + std::fmt::Debug,
    <B as HttpBody>::Data: Send,
{
    /// Create a proxy to a PMR2 instance served at the provided base `Uri`.
    ///
    /// This implementation assumes the provided `Uri` has the scheme of `http` and that no query is
    /// provided as that will be ignored.
    pub fn new<T, P>(base: T, cache_dir: P, permits: usize) -> Result<Self, std::io::Error>
    where
        P: Into<Utf8PathBuf>,
        T: TryInto<Uri>,
        <T as TryInto<Uri>>::Error: std::fmt::Debug,
    {
        let proxy = Proxy::new(base);
        let cache_dir = cache_dir.into();

        let cache_data = cache_dir.join(DATA);
        let cache_service = cache_dir.join(SERVICE);
        create_dir_all(&cache_data)?;
        create_dir_all(&cache_service)?;
        let serve_dir_data = ServeDir::new(&cache_data);
        let serve_dir_service = ServeDir::new(&cache_service);

        let semaphore = Semaphore::new(permits);
        let inner = Arc::new(GatewayInner {
            proxy,
            semaphore,
            cache_data,
            cache_service,
            serve_dir_data,
            serve_dir_service,
        });
        Ok(Self { inner })
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
        Box::pin(async move {
            let strategy = Strategy::new(service, req);
            #[cfg(feature = "trace")]
            tracing::debug!("strategy={strategy:?}");

            // TODO This look suspiciously like something that could be implemented by `Strategy` itself,
            // or that it could implement Future based on this.  For now just prototype this out and see
            // how that might be done later.
            match strategy {
                Strategy::Cache { path, serve_dir, mut proxy, request } => {
                    // call serve_dir and this be the fallback?
                    // need to figure out how to best structure this
                    proxy.call(request).await
                }
                Strategy::CacheWebService { path, serve_dir, mut proxy, request } => {
                    // call serve_dir and this be the fallback?
                    // need to figure out how to best structure this
                    // for now passthrough this.
                    proxy.call(request).await
                }
                Strategy::Semaphore { mut proxy, request } => {
                    // wait on semaphore, if unacquired within timeout generate a 504 response.
                    proxy.call(request).await
                }
                Strategy::Passthru { mut proxy, request } => {
                    proxy.call(request).await
                }
                Strategy::NotFound => {
                    // TODO Build a themed response off this.
                    Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from("<html><body><h1>404 Not Found</h1></body></html>"))
                        .expect("statically constructed response shouldn't error")
                    )
                }
            }
        })
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
                .unwrap_or_else(|_| {
                    // TODO Build a themed response off this.
                    Response::builder()
                        .status(StatusCode::SERVICE_UNAVAILABLE)
                        .header(header::CONTENT_TYPE, "text/html")
                        .body(Body::from("<html><body><h1>503 Servce Unavailable</h1></body></html>"))
                        .expect("statically constructed response shouldn't error")
                });
            Ok(resp)
        })
    }
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
fn resolve_path(path: &str) -> Utf8PathBuf {
    let mut p = path.split("/")
        .map(|s| {
            if s == "index.html" {
                "index_html".to_owned()
            } else {
                format!("{s}")
            }
        })
        .collect::<Utf8PathBuf>();
    p.push("index.html");
    p
}

#[cfg(test)]
mod test {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn proxy_patch_request() {
        let tempdir = TempDir::new().unwrap();
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
        assert_eq!(resolve_path("/"), "index.html");
        assert_eq!(resolve_path("/test"), "test/index.html");
        assert_eq!(resolve_path("/test/"), "test/index.html");
        assert_eq!(resolve_path("/test/target"), "test/target/index.html");
        assert_eq!(resolve_path("/test//target"), "test/target/index.html");
        assert_eq!(resolve_path("/test///target"), "test/target/index.html");
        assert_eq!(resolve_path("/index_html"), "index_html/index.html");
        assert_eq!(resolve_path("/index.html"), "index_html/index.html");
        assert_eq!(resolve_path("/test/index_html"), "test/index_html/index.html");
        assert_eq!(resolve_path("/test/index.html"), "test/index_html/index.html");

        // Plone usually has the `/view` suffix being the same as the underlying resource, but for now this
        // resolver ignores this issue as the optimization can be done by the underlying service.
        assert_eq!(resolve_path("/welcome"), "welcome/index.html");
        assert_eq!(resolve_path("/welcome/view"), "welcome/view/index.html");
    }
}
