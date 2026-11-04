use axum::{
    body::{Body, HttpBody},
    extract::OriginalUri,
    http::{
        header::{self, HeaderName, HeaderValue},
        Method, Request, Response, StatusCode,
    },
};
use camino::Utf8PathBuf;
use cookie::Cookie;
use futures::TryStreamExt;
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
    path::CachePathBuf,
    response::{http_404, http_500, http_503, http_504},
    Gateway, Proxy,
};

const HEADER_SEP: &str = ": ";

#[derive(Debug)]
pub struct CacheStrategy<B> {
    cache_dir: Utf8PathBuf,
    invalidate_cache: bool,
    path: CachePathBuf,
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
            // This will be required for the `ServeDir` later, as some of these services have been
            // nested and `ServeDir` is set up with the original root assumptions, so the original
            // will be required.
            let request_uri = self.semaphore_strategy.request.extensions()
                .get::<OriginalUri>()
                .map(|u| u.0.clone())
                .unwrap_or_else(|| self.semaphore_strategy.request.uri().clone());

            // call serve_dir and this be the fallback?
            // need to figure out how to best structure this
            // Assumes this is a `GET` request so body could be ignored later.
            let (mut parts, body) = self.semaphore_strategy.request.into_parts();
            let cache_path = self.cache_dir.join(&self.path);
            #[cfg(feature = "trace")]
            tracing::debug!("cache_path={cache_path}");

            let cache_headers_path = self.path.headers().map(|headers| self.cache_dir.join(headers));

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
                                        header::CONTENT_DISPOSITION,
                                        header::CONTENT_LANGUAGE,
                                        header::CONTENT_TYPE,
                                        header::LOCATION,
                                        header::VARY,
                                    ].iter() {
                                        if let Some(value) = parts.headers.get(header) {
                                            if let Ok(value) = value.to_str() {
                                                let _ = file.write(header.as_str().as_bytes()).await;
                                                let _ = file.write(HEADER_SEP.as_bytes()).await;
                                                let _ = file.write(value.as_bytes()).await;
                                                let _ = file.write(&[b'\n']).await;
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
            parts.uri = request_uri;
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
                        let mut split = line.splitn(2, HEADER_SEP);
                        let header = split
                            .next()
                            .map(|s| HeaderName::from_bytes(s.as_ref()));
                        let value = split
                            .next()
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
                    #[cfg(feature = "trace")]
                    tracing::warn!("semaphore is closed");
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
pub struct InvalidateCacheStrategy<B> {
    gateway: Gateway<B>,
    request: Request<B>,
    path: CachePathBuf,
}

impl<B> IntoFuture for InvalidateCacheStrategy<B>
where
    B: Send + HttpBody + Unpin + std::fmt::Debug + 'static,
    <B as HttpBody>::Data: Send,
    <B as HttpBody>::Error: std::error::Error + Send + Sync,
{
    type Output = Result<Response<Body>, Infallible>;
    type IntoFuture = Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send + 'static>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            match self.gateway.inner.proxy.clone().call(self.request).await {
                Err(_) => {
                    #[cfg(feature = "trace")]
                    tracing::debug!("cannot invalidate cache without a successful response");
                    Ok(http_503())
                }
                Ok(resp) => {
                    // Check that the status for this request is a success or a redirection, and defer to
                    // the gateway to determine how to action (or inaction) the cache invalidation.
                    let status = resp.status();
                    if status.is_success() || status.is_redirection() {
                        #[cfg(feature = "trace")]
                        tracing::info!("invalidating cache for this item path={}", self.path);
                        self.gateway.invalidate_cached_path(&self.path).await;
                    }
                    // All responses are returned as is.
                    Ok(resp)
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
    /// Invalidate the cache specified and pass through the request to the underlying service unhindered.
    InvalidateCache(InvalidateCacheStrategy<B>),
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
            Self::InvalidateCache(strategy) => {
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
        let request_uri = request.extensions()
            .get::<OriginalUri>()
            .map(|u| u.0.clone())
            .unwrap_or_else(|| request.uri().clone());

        let has_session = if let Some(Ok(cookie)) = request
            .headers()
            .get(header::COOKIE)
            .map(HeaderValue::to_str)
        {
            Cookie::split_parse(cookie).any(|r| r.map(|c| c.name() == "__ac").unwrap_or(false))
        } else {
            false
        };
        let has_authorization = request.headers().get(header::AUTHORIZATION).is_some();

        if has_session || has_authorization {
            // Ideally there should be an additional session management be implemented here, but leaving
            // that blind will still work, just that more requests may potentially be not served by the
            // cache.
            let request_path = request_uri.path();
            if let Some(path) = CachePathBuf::new(request_path)
                .and_then(|path| {
                    if request.method() == Method::POST {
                        #[cfg(feature = "trace")]
                        tracing::info!("invalidation: request_path={request_path}");
                        // This assumes the gateway is set up with the path defined with this in mind.
                        // This should typically work as all endpoints of interest have a single level
                        // component that affect the parent of any given endpoints.
                        path.parent()
                    } else if request_uri.path().ends_with("/content_status_modify") {
                        // Unfortunately, there are Plone end points that are GET that are not idempotent
                        // as it will cause changes.  Naturally those are assumed to trigger the
                        // invalidation.
                        #[cfg(feature = "trace")]
                        tracing::info!("invalidation: request_path={request_path}");
                        path.parent()
                    } else {
                        None
                    }
                })
            {
                #[cfg(feature = "trace")]
                tracing::info!("invalidation: path={path}");
                return Strategy::InvalidateCache(InvalidateCacheStrategy {
                    gateway: gateway.clone(),
                    path,
                    request,
                });
            } else {
                return Strategy::Passthru(PassthruStrategy {
                    proxy: gateway.inner.proxy.clone(),
                    request,
                });
            }
        }

        if request.method() == Method::GET {
            if let Some(path) = CachePathBuf::new(request_uri.path()) {
                if let Some(query) = request_uri.query() {
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

                // First, based on the implemented content negotiation, determine which cache to use.
                let accept = request
                    .headers()
                    .get(header::ACCEPT)
                    .map(HeaderValue::to_str)
                    .and_then(Result::ok)
                    .unwrap_or("");
                let (cache_dir, serve_dir) = gateway.negotiate_accept_cache(accept);

                // Workspaces hits are assumed to be heavy.
                // TODO Not all workspace endpoints should be cached.
                let request_path = request_uri.path();
                let (semaphore, semaphore_timeout) = if request_path.starts_with("/workspace") && (
                    request_path.contains("/download/") ||
                    request_path.contains("/@@download/") ||
                    request_path.contains("/rawfile/") ||
                    request_path.contains("/@@rawfile/") ||
                    request_path.contains("/archive/") ||
                    request_path.contains("/@@archive/")
                ) {
                    (
                        gateway.inner.heavy_semaphore.clone(),
                        gateway.inner.heavy_semaphore_timeout,
                    )
                } else {
                    (
                        gateway.inner.cache_semaphore.clone(),
                        gateway.inner.cache_semaphore_timeout,
                    )
                };

                return Strategy::Cache(CacheStrategy {
                    cache_dir,
                    invalidate_cache: false,
                    path,
                    serve_dir,
                    semaphore_strategy: SemaphoreStrategy {
                        proxy: gateway.inner.proxy.clone(),
                        semaphore,
                        semaphore_timeout,
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
