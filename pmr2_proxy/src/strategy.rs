use axum::{
    body::{Body, HttpBody},
    http::{
        header::{self, HeaderName, HeaderValue},
        Method, Request, Response, StatusCode,
    },
};
use camino::Utf8PathBuf;
use cookie::Cookie;
use futures::TryStreamExt;
use percent_encoding::percent_decode;
use std::{
    convert::Infallible,
    fs::create_dir_all,
    future::Future,
    pin::Pin,
    sync::Arc,
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

use crate::{
    response::{http_404, http_500, http_503, http_504},
    Gateway, Proxy,
};

#[derive(Debug)]
pub struct CacheStrategy<B> {
    cache_dir: Utf8PathBuf,
    invalidate_cache: bool,
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

            // Not a file, clone the parts, pop out the FOUND hader, and get the full content from the
            // undering to be cached.
            // Also do the same if the cache is to be invalidated.
            if !cache_path.is_file() || self.invalidate_cache {
                #[cfg(feature = "trace")]
                if !cache_path.is_file() {
                    tracing::debug!("cache_path={cache_path} not file");
                } else {
                    tracing::debug!("cache_path={cache_path} to be invalidated");
                }

                let mut parts = parts.clone();
                parts.headers.remove(header::RANGE);
                self.semaphore_strategy.request = Request::from_parts(parts, body);

                match self.semaphore_strategy.into_future().await {
                    Err(_) => {
                        #[cfg(feature = "trace")]
                        tracing::info!("proxy.call");
                        return Ok(http_503());
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
                            return Ok(http_500());
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
                                return Ok(http_500());
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
                            return Ok(http_500());
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
                    Ok(http_500())
                }
                Err(_) => {
                    // Timeout acquiring the semaphore.
                    Ok(http_504())
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
                    Ok(http_404())
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
                if let Some(query) = request.uri().query() {
                    if query != "" {
                        // GET requests with query strings are not cached, and simply passed through.
                        // TODO Figure out how to derive a path for the specific query string, if this
                        // proved to be necessary.
                        return Strategy::Semaphore(SemaphoreStrategy {
                            proxy: gateway.inner.proxy.clone(),
                            semaphore: gateway.inner.passthru_semaphore.clone(),
                            semaphore_timeout: gateway.inner.passthru_semaphore_timeout,
                            request,
                        });
                    }
                }

                // For the meantime responses from the web service could be cached if they are GET and
                // without query strings.
                if let Some(Ok(accept)) = request
                    .headers()
                    .get(header::ACCEPT)
                    .map(HeaderValue::to_str)
                {
                    if accept == "application/vnd.physiome.pmr2.json.1" {
                        return Strategy::CacheWebService(CacheStrategy {
                            cache_dir: gateway.inner.cache_service.clone(),
                            invalidate_cache: false,
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
                    invalidate_cache: false,
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
