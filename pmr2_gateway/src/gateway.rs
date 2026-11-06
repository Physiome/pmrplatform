use axum::{
    body::{Body, HttpBody},
    http::{Request, Response, Uri},
};
use camino::Utf8PathBuf;
use headers_accept::Accept;
use mediatype::MediaType;
use std::{
    convert::Infallible,
    fs::create_dir_all,
    future::Future,
    pin::Pin,
    str::FromStr,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::sync::Semaphore;
use tower::Service;
use tower_http::services::fs::ServeDir;

use crate::{
    path::CachePathBuf,
    Proxy, Strategy,
};

const DATA: &str = "data";
const SERVICE_0: &str = "json_0";
const SERVICE_1_1: &str = "json_1_1";
const SERVICE_1_2: &str = "json_1_2";

#[derive(Debug)]
struct Cache {
    path: Utf8PathBuf,
    service: ServeDir,
}

impl Cache {
    fn new(path: Utf8PathBuf) -> std::io::Result<Self> {
        create_dir_all(&path)?;
        let service = ServeDir::new(&path)
            .redirect_to_trailing_slash(false);
        Ok(Self { path, service })
    }

    fn to_parts(&self) -> (Utf8PathBuf, ServeDir) {
        (self.path.clone(), self.service.clone())
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
    heavy_permits: usize,
    heavy_timeout: Duration,
    passthru_permits: usize,
    passthru_timeout: Duration,
}

impl Default for GatewayBuilder {
    fn default() -> Self {
        Self {
            cache_dir: ".".into(),
            cache_permits: 3,
            cache_timeout: Duration::from_secs(6),
            heavy_permits: 1,
            heavy_timeout: Duration::from_secs(2),
            passthru_permits: 3,
            passthru_timeout: Duration::from_secs(6),
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

    /// Specify the number of permits allowed for resources that will result in a heavy CPU load.
    pub fn heavy_permits(mut self, heavy_permits: usize) -> Self {
        self.heavy_permits = heavy_permits;
        self
    }

    /// Specify the timeout in seconds that may be waited on the heavy permit.
    pub fn heavy_timeout(mut self, heavy_timeout: u64) -> Self {
        self.heavy_timeout = Duration::from_secs(heavy_timeout);
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

        let cache_data = Cache::new(self.cache_dir.join(DATA))?;
        let cache_service_0 = Cache::new(self.cache_dir.join(SERVICE_0))?;
        let cache_service_1_1 = Cache::new(self.cache_dir.join(SERVICE_1_1))?;
        let cache_service_1_2 = Cache::new(self.cache_dir.join(SERVICE_1_2))?;

        let passthru_semaphore = Arc::new(Semaphore::new(self.passthru_permits));
        let passthru_semaphore_timeout = self.passthru_timeout;
        let heavy_semaphore = Arc::new(Semaphore::new(self.heavy_permits));
        let heavy_semaphore_timeout = self.heavy_timeout;
        let cache_semaphore = Arc::new(Semaphore::new(self.cache_permits));
        let cache_semaphore_timeout = self.cache_timeout;

        let inner = Arc::new(GatewayInner {
            proxy,
            passthru_semaphore,
            passthru_semaphore_timeout,
            heavy_semaphore,
            heavy_semaphore_timeout,
            cache_data,
            cache_service_0,
            cache_service_1_1,
            cache_service_1_2,
            cache_semaphore,
            cache_semaphore_timeout,
        });
        Ok(Gateway {
            inner,
            invalidation_targets: None,
            invalidation_method: InvalidationMethod::Noop,
        })
    }
}

#[derive(Debug)]
pub(crate) struct GatewayInner<B> {
    pub(crate) proxy: Proxy<B>,

    // These are for the direct access that won't be cached.
    pub(crate) passthru_semaphore: Arc<Semaphore>,
    pub(crate) passthru_semaphore_timeout: Duration,

    // These are for heavy resources.
    pub(crate) heavy_semaphore: Arc<Semaphore>,
    pub(crate) heavy_semaphore_timeout: Duration,

    // These are to ensure access to resources that have yet to be cached will not overload the underlying.
    pub(crate) cache_semaphore: Arc<Semaphore>,
    pub(crate) cache_semaphore_timeout: Duration,
    cache_data: Cache,
    cache_service_0: Cache,
    cache_service_1_1: Cache,
    cache_service_1_2: Cache,

}

#[derive(Clone, Copy, Debug)]
enum InvalidationMethod {
    Noop,
    Single,
    Recursive,
}

#[derive(Debug)]
pub struct Gateway<B> {
    pub(crate) inner: Arc<GatewayInner<B>>,
    // I really dislike how I stuck this guy here.  What I wanted was a way to convey what should be removed
    // if invalidation is required by whatever strategy the request was made.  Originally a gateway is
    // uniform across the whole application (within the `Router`), but a convenient place to store definition
    // is directly with the underlying service.  This guy is the result.
    //
    // Ultimately the strategy will make use of this definition through the provided method in this gateway,
    // where the items associated with the request will be removed as per the method along with whatever other
    // targets be removed.  This is so that the gateway assigned to the fallback service (i.e. at the root
    // level) may have the `Noop` removal method such that no invalidation will be triggered (e.g. the login
    // and the various query endpoints).
    //
    // That all being said, this has the advantage of having the definition of what can be invalidated be
    // defined at the `Router` level along with the routes.
    pub(crate) invalidation_targets: Option<Arc<[CachePathBuf]>>,
    invalidation_method: InvalidationMethod,
}

impl<B> Clone for Gateway<B> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            invalidation_targets: self.invalidation_targets.clone(),
            invalidation_method: self.invalidation_method,
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

impl<B> Gateway<B> {
    /// Clone this gateway with the provided invalidation targets set to the provided argument.
    ///
    /// This will result in the `InvalidateCacheStrategy` deleting all targets non-recursively as specified.
    pub fn with_invalidation_targets<T>(
        &self,
        invalidation_targets: impl Iterator<Item = T>,
    ) -> Gateway<B>
    where
        T: Into<String>,
    {
        let mut gateway = self.clone();
        gateway.invalidation_targets = Some(invalidation_targets
            .map(Into::into)
            .filter_map(CachePathBuf::new)
            .collect::<Vec<_>>()
            .into()
        );
        gateway
    }

    /// Clone this gateway and specify the invalidation method to be no-op for the clone.
    ///
    /// This will result in the `InvalidateCacheStrategy` not trigger any deletion of the specified cached
    /// item.
    pub fn with_noop_invalidation_method(&self) -> Gateway<B> {
        let mut gateway = self.clone();
        gateway.invalidation_method = InvalidationMethod::Noop;
        gateway
    }

    /// Clone this gateway and specify the invalidation method to be single for the clone.
    ///
    /// This will result in the `InvalidateCacheStrategy` to delete the specified cached item.
    pub fn with_single_invalidation_method(&self) -> Gateway<B> {
        let mut gateway = self.clone();
        gateway.invalidation_method = InvalidationMethod::Single;
        gateway
    }

    /// Clone this gateway and specify the invalidation method to be recursive for the clone.
    ///
    /// This will result in the `InvalidateCacheStrategy` to delete the specified cached item recursively,
    /// i.e. including all content subpath to the path in the original request.
    pub fn with_recursive_invalidation_method(&self) -> Gateway<B> {
        let mut gateway = self.clone();
        gateway.invalidation_method = InvalidationMethod::Recursive;
        gateway
    }

    /// Negotiate for the correct cache based on the provided accept header value.
    pub fn negotiate_accept_cache(&self, accept: &str) -> (Utf8PathBuf, ServeDir) {
        match Handler::from_str(accept).expect("this is infallible") {
            Handler::Ignore => self.inner.cache_data.to_parts(),
            Handler::V0 => self.inner.cache_service_0.to_parts(),
            Handler::V1_1 => self.inner.cache_service_1_1.to_parts(),
            Handler::V1_2 => self.inner.cache_service_1_2.to_parts(),
        }
    }

    /// Invalidate the cache targets stored for this instance of Gateway.
    pub async fn invalidate_cache(&self) {
        self.invalidate_cached_paths(
            self.invalidation_targets
                .as_deref()
                .into_iter()
                .flatten()
        )
        .await
    }

    /// Invalidate the provided paths from the cache.
    async fn invalidate_cached_paths<'a>(&self, paths: impl Iterator<Item = &'a CachePathBuf>) {
        let services = [
            &self.inner.cache_data,
            &self.inner.cache_service_0,
            &self.inner.cache_service_1_1,
            &self.inner.cache_service_1_2,
        ];
        for path in paths {
            for service in services.iter() {
                path.invalidate_from(&service.path).await;
            }
        }
    }

    /// Invalidate the provided path as an item, if this gateway was set up with some invalidation targets.
    /// This will remove the specified invalidation targets, and will also recursively delete all cached
    /// content associated with the provided cached path.
    ///
    /// Otherwise if `None` was specified, this is an no-op.
    pub(crate) async fn invalidate_cached_path<'a>(&self, path: &'a CachePathBuf) {
        let services = [
            &self.inner.cache_data,
            &self.inner.cache_service_0,
            &self.inner.cache_service_1_1,
            &self.inner.cache_service_1_2,
        ];
        match self.invalidation_method {
            InvalidationMethod::Noop => {}
            InvalidationMethod::Single => {
                for service in services.iter() {
                    path.invalidate_from(&service.path).await;
                }
            }
            InvalidationMethod::Recursive => {
                for service in services.iter() {
                    path.invalidate_recursive_from(&service.path).await;
                }
            }
        }
        self.invalidate_cache().await;
    }

    /// Checks whether the provided item is cached in any of the caches.
    pub(crate) async fn is_cached_path<'a>(&self, path: &'a CachePathBuf) -> bool {
        let services = [
            &self.inner.cache_data,
            &self.inner.cache_service_0,
            &self.inner.cache_service_1_1,
            &self.inner.cache_service_1_2,
        ];
        services.iter()
            .any(|service| path.exists_in(&service.path))
    }
}

#[derive(Debug, PartialEq)]
enum Handler {
    Ignore,
    V0,
    V1_1,
    V1_2,
}

impl FromStr for Handler {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let any = MediaType::parse("*/*")
            .expect("provided static value should parse");
        let v0 = MediaType::parse("application/vnd.physiome.pmr2.json.0")
            .expect("provided static value should parse");
        let v1 = MediaType::parse("application/vnd.physiome.pmr2.json.1")
            .expect("provided static value should parse");
        let v1_1 = MediaType::parse("application/vnd.physiome.pmr2.json.1; version=1")
            .expect("provided static value should parse");
        let v1_2 = MediaType::parse("application/vnd.physiome.pmr2.json.1; version=2")
            .expect("provided static value should parse");
        let application_json = MediaType::parse("application/json")
            .expect("provided static value should parse");
        let mut media_types = [
            &any,
            &v1,
            &v1_1,
            &v1_2,
            &v0,
            &application_json,
        ]
        .into_iter();
        let result = if let Ok(accept) = Accept::from_str(s) {
            match accept.negotiate(&mut media_types) {
                Some(v) if *v == any => Self::Ignore,
                Some(v) if *v == v0 => Self::V0,
                Some(v) if *v == v1 => Self::V1_1,
                Some(v) if *v == v1_1 => Self::V1_1,
                Some(v) if *v == v1_2 => Self::V1_2,
                Some(v) if *v == application_json => Self::V1_1,
                _ => Self::Ignore,
            }
        } else {
            Self::Ignore
        };
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn negotiate() -> Result<(), Infallible> {
        assert_eq!(Handler::from_str("")?, Handler::Ignore);
        assert_eq!(Handler::from_str("*/*")?, Handler::Ignore);
        assert_eq!(Handler::from_str("text/html")?, Handler::Ignore);
        assert_eq!(Handler::from_str("application/json")?, Handler::V1_1);
        assert_eq!(Handler::from_str("application/vnd.physiome.pmr2.json.0")?, Handler::V0);
        assert_eq!(Handler::from_str("application/vnd.physiome.pmr2.json.1")?, Handler::V1_1);
        assert_eq!(Handler::from_str("application/vnd.physiome.pmr2.json.1; version=1")?, Handler::V1_1);
        assert_eq!(Handler::from_str("application/vnd.physiome.pmr2.json.1; version=2")?, Handler::V1_2);
        // This isn't documented in PMR2 but this should still work.
        assert_eq!(
            Handler::from_str("*/*, application/vnd.physiome.pmr2.json.1; version=2; q=1")?,
            Handler::V1_2,
        );
        Ok(())
    }
}
