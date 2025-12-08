use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use axum_login::{AuthManager, AuthnBackend};
use pmrcore::ac::session::SessionToken;
use http::{header, Request, Response};
use tower_cookies::CookieManager;
use tower_layer::Layer;
use tower_service::Service;
use tower_sessions::{
    session::Id,
    service::CookieController,
    Expiry,
    Session,
    SessionStore,
    SessionManager,
    SessionManagerLayer,
};

use crate::axum_login::{
    BearerTokenManager,
    BearerTokenManagerConfig,
    BearerTokenManagerLayer,
};

impl<S, Store> BearerTokenManager<S, Store>
where
    Store: SessionStore,
{
    pub fn new(inner: S, store: Store) -> Self {
        Self {
            inner,
            store: store.into(),
            config: BearerTokenManagerConfig::default(),
        }
    }
}

impl<ReqBody, ResBody, S, Store: SessionStore> Service<Request<ReqBody>> for BearerTokenManager<S, Store>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Default + Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let store = self.store.clone();
        let config = self.config.clone();

        // Because the inner service can panic until ready, we need to ensure we only
        // use the ready service.
        //
        // See: https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(
            async move {
                let value = req.headers()
                    .get(header::AUTHORIZATION);
                if let Some(value) = value {
                    if let Ok(authorization) = value.to_str() {
                        match authorization.split_once(' ') {
                            Some((name, token)) if name == "Bearer" => {
                                if let Ok(session_id) = token.parse::<Id>() {
                                    // Only override the session when provided with a valid bearer token.
                                    let session = Session::new(Some(session_id), store, config.expiry);
                                    req.extensions_mut().insert(session);
                                }
                            }
                            _ => (),
                        }
                    }
                } else {
                    if Some(req.uri().to_string().as_ref()) == config.new_bearer_endpoint {
                        let session = Session::new(Some(Id::default()), store, config.expiry);
                        req.extensions_mut().insert(session);
                    }
                }
                inner.call(req).await
            }
        )
    }
}

impl<
    Store: SessionStore,
    C: CookieController,
    Backend: AuthnBackend,
> BearerTokenManagerLayer<Store, C, Backend> {
    pub fn new(
        store: Store,
        backend: Backend,
        session_manager_layer: SessionManagerLayer<Store, C>,
    ) -> Self {
        let config = BearerTokenManagerConfig::default();

        Self {
            store: Arc::new(store),
            config,
            session_manager_layer,
            backend,
            data_key: None,
        }
    }

    pub fn with_expiry(mut self, expiry: Option<Expiry>) -> Self {
        self.config.expiry = expiry;
        self
    }

    pub fn with_data_key(mut self, data_key: &'static str) -> Self {
        self.config.data_key = Some(data_key);
        self
    }

    pub fn with_new_bearer_endpoint(mut self, new_bearer_endpoint: &'static str) -> Self {
        self.config.new_bearer_endpoint = Some(new_bearer_endpoint);
        self
    }
}

impl<
    S,
    Store: SessionStore,
    C: CookieController,
    Backend: AuthnBackend,
> Layer<S> for BearerTokenManagerLayer<Store, C, Backend> {
    type Service = CookieManager<SessionManager<BearerTokenManager<AuthManager<S, Backend>, Store>, Store, C>>;

    fn layer(&self, inner: S) -> Self::Service {
        let login_manager = AuthManager::new(
            inner,
            self.backend.clone(),
            self.data_key.unwrap_or("axum-login.data"),
        );
        let bearer_manager = BearerTokenManager {
            inner: login_manager,
            store: self.store.clone(),
            config: self.config.clone(),
        };

        self.session_manager_layer
            .layer(bearer_manager)
    }
}
