use std::sync::Arc;
use ::axum_login::AuthnBackend;
use tower_sessions::{
    service::CookieController,
    Expiry,
    SessionStore,
    SessionManagerLayer,
};

pub struct Credentials {
    pub authorization: Authorization,
    pub origin: String,
}

#[non_exhaustive]
pub enum Authorization {
    LoginPassword(String, String),
    Token(String),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BearerTokenManagerConfig {
    pub(crate) expiry: Option<Expiry>,
    pub(crate) data_key: Option<&'static str>,
}

#[derive(Clone)]
pub struct BearerTokenManager<S, Store> {
    inner: S,
    store: Arc<Store>,
    config: BearerTokenManagerConfig,
}

#[derive(Debug, Clone)]
pub struct BearerTokenManagerLayer<
    Store: SessionStore,
    C: CookieController,
    Backend: AuthnBackend,
> {
    store: Arc<Store>,
    config: BearerTokenManagerConfig,
    session_manager_layer: SessionManagerLayer<Store, C>,
    backend: Backend,
    data_key: Option<&'static str>,
}

mod impls;
