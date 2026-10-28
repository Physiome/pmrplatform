use axum::{
    body::{Body, HttpBody},
    http::{Request, Response, Uri},
};
use camino::Utf8PathBuf;
use std::{
    convert::Infallible,
    fs::create_dir_all,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::sync::Semaphore;
use tower::Service;
use tower_http::services::fs::ServeDir;

use crate::{Proxy, Strategy};

const DATA: &str = "data";
const SERVICE: &str = "webservice";

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
pub(crate) struct GatewayInner<B> {
    pub(crate) proxy: Proxy<B>,

    // These are for the direct access that won't be cached.
    pub(crate) passthru_semaphore: Arc<Semaphore>,
    pub(crate) passthru_semaphore_timeout: Duration,

    // These are to ensure access to resources that have yet to be cached will not overload the underlying.
    pub(crate) cache_semaphore: Arc<Semaphore>,
    pub(crate) cache_semaphore_timeout: Duration,
    pub(crate) cache_data: Utf8PathBuf,
    pub(crate) cache_service: Utf8PathBuf,

    // The services for the relevant underlying data.
    pub(crate) serve_dir_data: ServeDir,
    pub(crate) serve_dir_service: ServeDir,
}

#[derive(Debug)]
pub struct Gateway<B> {
    pub(crate) inner: Arc<GatewayInner<B>>,
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
