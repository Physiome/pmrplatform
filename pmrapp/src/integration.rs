use axum::{
    extract::{
        Extension,
        FromRequestParts,
        Request,
    },
    handler::Handler,
    http::{
        Method,
        Uri,
        header::{self, HeaderValue},
        uri::{
            Authority,
            PathAndQuery,
            Scheme,
        },
    },
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        delete,
        get,
        patch,
        post,
        put,
    },
};
use leptos::server_fn::axum::server_fn_paths;
use leptos_axum::handle_server_fns;
use axum_login_bearer::BearerTokenAuthManagerLayer;
use pmrcore::web::Source;
use pmrctrl::platform::Platform;
use std::pin::Pin;
use time::Duration;
use tower::{
    Layer,
    ServiceBuilder,
    util::{MapRequest, MapRequestLayer},
};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
#[cfg(feature = "utoipa")]
use utoipa::OpenApi;
use crate::{
    error::AppError,
    exposure::api::WIZARD_FIELD_ROUTE,
    server::{
        exposure::{
            aliased_exposure_archive_zip,
            exposure_file_data,
            exposure_file_safe_html,
            wizard_field_update,
        },
        index,
        workspace::{
            collection_json_workspace,
            aliased_workspace_archive_tgz,
            aliased_workspace_archive_zip,
            aliased_workspace_rawfile_download,
            workspace_archive_tgz,
            workspace_archive_zip,
            workspace_rawfile_download,
        },
    },
};

mod private {
    pub trait Sealed {}
    impl<S> Sealed for axum::Router<S> {}
}

fn reroute_collection_json<B>(req: &mut Request<B>) {
    // naively resolve our header
    if req.headers().get("accept") == Some(&HeaderValue::from_static("application/vnd.physiome.pmr2.json.1")) {
        // TODO this should be defined as a constant for use with building the router
        let prefix = "/collection_json";
        let mut parts = req.uri().clone().into_parts();
        parts.path_and_query = parts.path_and_query
            .map(|v| PathAndQuery::try_from(format!("{prefix}{v}")).expect("original parsed fine"));
        *req.uri_mut() = Uri::from_parts(parts).expect("original parts should be valid");
    }
}

// Before the request is handed off down to the axum app, ensure whatever we may need are processed.
fn before_handle_request<B>(mut req: Request<B>) -> Request<B> {
    // Grab the original requested uri before the rerouting.
    let mut uri_parts = req.uri().clone().into_parts();
    let host = req
        .headers()
        .get("host")
        .cloned()
        .unwrap_or_else(|| HeaderValue::from_static("localhost"));
    let authority = Authority::try_from(host.as_bytes())
        .unwrap_or_else(|_| Authority::from_static("localhost"));
    // FIXME how to determine this assumption
    uri_parts.scheme = Some(if authority.host() == "localhost" {
        Scheme::HTTP
    } else {
        Scheme::HTTPS
    });
    uri_parts.authority = Some(authority);
    req.extensions_mut().insert(Source(Uri::from_parts(uri_parts).expect("parts are valid")));
    reroute_collection_json(&mut req);
    req
}

/// Extension trait for [`axum::Router`] so it may be set up for serving of the PMR platform.
///
/// This trait is sealed; implementation by external package is not possible.
pub trait PmrAxumExt<S>: private::Sealed
where
    S: Clone + Send + Sync + 'static,
{
    /// This is to set up `get` routes for the pmr application where an additional error renderer may be
    /// provided.
    fn pmr_route_get<F, E, EFut, ERes, T>(self, path: &str, f: F, error_handler: E) -> Self
    where
        F: PmrHandlerExt<T, S>,
        E: FnOnce(Request, AppError) -> EFut + Clone + Send + Sync + 'static,
        EFut: Future<Output = ERes> + Send,
        ERes: IntoResponse + Send,
        T: 'static;

    /// Enable routing to the data-only (e.g. rawfile, archive) endpoints of PMR.
    ///
    /// Use this to set up the services provided by the `pmrapp::server` module.
    ///
    /// Additionally, the error handler must be provided.  The following setup simply return the `AppError`
    /// as-is, where its `IntoResponse` implementation will provide the default status code response.
    ///
    /// ```
    /// router.pmr_server_routes(|_, e| async { e })
    /// ```
    ///
    /// Please ensure that `.pmr_layers()` is called at some point after PMR routes have been added.
    fn pmr_server_routes<E, EFut, Res>(self, error_handler: E) -> Self
    where
        E: FnOnce(Request, AppError) -> EFut + Clone + Send + Sync + 'static,
        EFut: Future<Output = Res> + Send,
        Res: IntoResponse + Send;

    /// Set up the full API routes related to PMR.
    ///
    /// Use this to set up the services provided by the `pmrapp::server` module along with the services
    /// implemented as Leptos server functions.
    ///
    /// Please ensure that `.layers()` is called at some point after PMR routes have been added.
    fn pmr_routes< E, EFut, Res>(self, error_handler: E) -> Self
    where
        E: FnOnce(Request, AppError) -> EFut + Clone + Send + Sync + 'static,
        EFut: Future<Output = Res> + Send,
        Res: IntoResponse + Send;

    /// Set up the layers required by PMR routes.
    ///
    /// A [`Platform`] must be provided.
    fn pmr_layers(self, platform: Platform, cors_allow_origin: Option<AllowOrigin>) -> Self;

    /// Setup the map requests required for full PMR functionality
    ///
    /// This ensures the request is processed to provide additional `Extension` items required, and that
    /// conversion to `Accept` header routes are provided.
    fn pmr_map_request<Body>(self) -> MapRequest<Self, fn(Request<Body>) -> Request<Body>>
        where Self: Sized;
}

impl<S> PmrAxumExt<S> for axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn pmr_route_get<F, E, EFut, ERes, T>(self, path: &str, f: F, error_handler: E) -> Self
    where
        F: PmrHandlerExt<T, S>,
        E: FnOnce(Request, AppError) -> EFut + Clone + Send + Sync + 'static,
        EFut: Future<Output = ERes> + Send,
        ERes: IntoResponse + Send,
        T: 'static,
    {
        self.without_v07_checks()
            .route(path, get(PmrHandler::new(f, error_handler)))
    }

    fn pmr_server_routes<E, EFut, Res>(self, error_handler: E) -> Self
    where
        E: FnOnce(Request, AppError) -> EFut + Clone + Send + Sync + 'static,
        EFut: Future<Output = Res> + Send,
        Res: IntoResponse + Send,
    {
        let router = self
            .without_v07_checks()
            // TODO the path should be constructed from a known list, so that rewriting only happens
            // to this route only if it exists.
            .route("/api/exposure/{e_id}/{ef_id}/{view_key}/{*path}", get(exposure_file_data))
            .route("/api/exposure/safe_html/{e_id}/{ef_id}/{view_key}/{*path}", get(exposure_file_safe_html))
            .route("/collection_json/workspace/", get(collection_json_workspace))

            .route("/api/exposure/{exposure_id}/download_zip", get(aliased_exposure_archive_zip))
            .pmr_route_get(
                "/exposure/{exposure_id}/:/download_zip",
                aliased_exposure_archive_zip,
                error_handler.clone(),
            )

            // These are duplicated to /api/ to keep the OpenAPI specification consistent, while
            // keeping the original in the event we will fall back to a fully integrated application.
            .pmr_route_get(
                "/workspace/{workspace_alias}/archive/{commit_id}/tgz",
                aliased_workspace_archive_tgz,
                error_handler.clone(),
            )
            .pmr_route_get(
                "/workspace/:/id/{workspace_id}/archive/{commit_id}/tgz",
                workspace_archive_tgz,
                error_handler.clone(),
            )
            .pmr_route_get(
                "/workspace/{workspace_alias}/archive/{commit_id}/zip",
                aliased_workspace_archive_zip,
                error_handler.clone(),
            )
            .pmr_route_get(
                "/workspace/:/id/{workspace_id}/archive/{commit_id}/zip",
                workspace_archive_zip,
                error_handler.clone(),
            )
            .pmr_route_get(
                "/workspace/{workspace_alias}/rawfile/{commit_id}/{*path}",
                aliased_workspace_rawfile_download,
                error_handler.clone(),
            )
            .pmr_route_get(
                "/workspace/:/id/{workspace_id}/rawfile/{commit_id}/{*path}",
                workspace_rawfile_download,
                error_handler.clone(),
            )
            .route("/api/workspace/{workspace_alias}/rawfile/{commit_id}/{*path}", get(aliased_workspace_rawfile_download))
            .route("/api/workspace/:/id/{workspace_id}/rawfile/{commit_id}/{*path}", get(workspace_rawfile_download))
            .route("/api/workspace/{workspace_alias}/archive/{commit_id}/tgz", get(aliased_workspace_archive_tgz))
            .route("/api/workspace/:/id/{workspace_id}/archive/{commit_id}/tgz", get(workspace_archive_tgz))
            .route("/api/workspace/{workspace_alias}/archive/{commit_id}/zip", get(aliased_workspace_archive_zip))
            .route("/api/workspace/:/id/{workspace_id}/archive/{commit_id}/zip", get(workspace_archive_zip))

            // Index routes
            .route("/api/citations", get(index::citations))
            .route("/api/citations/", get(index::citations))
            .route("/api/index", get(index::indexes))
            .route("/api/index/", get(index::indexes))
            .route("/api/index/{keyword}", get(index::terms))
            .route("/api/index/{keyword}/", get(index::terms))
            .route("/api/index/{keyword}/{term}", get(index::resources))
            .route("/api/index/{keyword}/{term}/", get(index::resources))
            .route("/api/search", post(index::resource_briefs))

            .route(WIZARD_FIELD_ROUTE, post(wizard_field_update));

        #[cfg(feature = "utoipa")]
        let router = router.merge(
            utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", crate::openapi::ApiDoc::openapi())
        );

        router
    }

    fn pmr_routes< E, EFut, Res>(self, error_handler: E) -> Self
    where
        E: FnOnce(Request, AppError) -> EFut + Clone + Send + Sync + 'static,
        EFut: Future<Output = Res> + Send,
        Res: IntoResponse + Send,
    {
        let mut router = self.pmr_server_routes(error_handler);
        for (path, method) in server_fn_paths() {
            router = router.route(
                path,
                match method {
                    Method::GET => get(handle_server_fns),
                    Method::POST => post(handle_server_fns),
                    Method::PUT => put(handle_server_fns),
                    Method::DELETE => delete(handle_server_fns),
                    Method::PATCH => patch(handle_server_fns),
                    _ => {
                        panic!(
                            "Unsupported server function HTTP method: \
                             {method:?}"
                        );
                    }
                },
            );
        }
        router
    }

    fn pmr_layers(self, platform: Platform, cors_allow_origin: Option<AllowOrigin>) -> Self {
        let session_store = MemoryStore::default();
        // let session_layer = SessionManagerLayer::new(session_store.clone())
        let session_layer = SessionManagerLayer::new(MemoryStore::default())
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::days(1)));

        let auth_service = ServiceBuilder::new()
            .layer(
                BearerTokenAuthManagerLayer::new(
                    session_store,
                    platform.ac_platform_clone(),
                )
                .with_session_manager_layer(session_layer)
                .with_bearer_token_endpoint("/api/bearer/from_login_password"),
            );

        let cors = CorsLayer::new()
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_origin(cors_allow_origin.unwrap_or([].into()));

        // TODO add an additional handler that will filter out the body
        // for status code 3xx to optimize output.
        self.layer(Extension(platform.clone()))
            .layer(auth_service)
            .layer(cors)
    }

    fn pmr_map_request<Body>(self) -> MapRequest<Self, fn(Request<Body>) -> Request<Body>> {
        MapRequestLayer::new(before_handle_request::<_> as fn(http::Request<Body>) -> http::Request<Body>)
            .layer(self)
    }
}

/// A wrapper around a function that is axum handler with an error handler.
///
/// Please refer to [`PmrHandler::new`].
#[derive(Clone)]
pub struct PmrHandler<F, E> {
    f: F,
    error: E,
}

impl<F, E> PmrHandler<F, E> {
    /// Create a wrapper around a `Handler` that returns a `Result<impl IntoResponse, AppError>` along
    /// with a function that takes a `Request` and `AppError` as arguments such that the `Err` arm of the
    /// handler's result may be processed into the intended error page for end-user consumption.
    pub fn new<EFut, ERes, T, S>(f: F, error: E) -> Self
    where
        F: PmrHandlerExt<T, S>,
        E: FnOnce(Request, AppError) -> EFut + Clone + Send + Sync + 'static,
        EFut: Future<Output = ERes> + Send,
        ERes: IntoResponse + Send,
    {
        Self { f, error }
    }
}

impl<F, E, EFut, S, ERes, T> Handler<T, S> for PmrHandler<F, E>
where
    F: PmrHandlerExt<T, S>,
    E: FnOnce(Request, AppError) -> EFut + Clone + Send + Sync + 'static,
    EFut: Future<Output = ERes> + Send,
    ERes: IntoResponse + Send,
    S: Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call(self, req: Request, state: S) -> Self::Future {
        Box::pin(async move {
            self.f.call(req, state, self.error).await
        })
    }
}

/// "Extension" trait for `Handler` that has a call method suitable for handlers fround in PMR.
///
/// This is not a strict extension as it basically has one concrete implementation provided by [`PmrHandler`],
/// where it encapsulate and implements `Handler` such that a common error page (for a given application) may
/// be produced from a common error handler.
pub trait PmrHandlerExt<T, S>: Clone + Send + Sync + Sized + 'static {
    type Future: Future<Output = Response> + Send + 'static;

    fn call<E, EFut, ERes>(self, req: Request, state: S, err_handler: E) -> Self::Future
    where
        E: FnOnce(Request, AppError) -> EFut + Clone + Send + Sync + 'static,
        EFut: Future<Output = ERes> + Send,
        ERes: IntoResponse + Send;
}

impl<F, Fut, S, Res, T1> PmrHandlerExt<(T1,), S> for F
where
    F: FnOnce(T1) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Res, AppError>> + Send,
    S: Send + Sync + 'static,
    Res: IntoResponse + Send,
    T1: FromRequestParts<S> + Send,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call<E, EFut, ERes>(self, req: Request, state: S, err_handler: E) -> Self::Future
    where
        E: FnOnce(Request, AppError) -> EFut + Clone + Send + Sync + 'static,
        EFut: Future<Output = ERes> + Send,
        ERes: IntoResponse + Send,
    {
        let (mut parts, body) = req.into_parts();
        Box::pin(async move {
            let t1 = match T1::from_request_parts(&mut parts, &state).await {
                Ok(value) => value,
                Err(rejection) => return rejection.into_response(),
            };

            let req = Request::from_parts(parts, body);

            match self(t1).await {
                Ok(r) => r.into_response(),
                Err(err) => {
                    err_handler(req, err)
                        .await
                        .into_response()
                }
            }
        })
    }
}

impl<F, Fut, S, Res, T1, T2> PmrHandlerExt<(T1, T2), S> for F
where
    F: FnOnce(T1, T2) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Res, AppError>> + Send,
    S: Send + Sync + 'static,
    Res: IntoResponse + Send,
    T1: FromRequestParts<S> + Send,
    T2: FromRequestParts<S> + Send,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call<E, EFut, ERes>(self, req: Request, state: S, err_handler: E) -> Self::Future
    where
        E: FnOnce(Request, AppError) -> EFut + Clone + Send + Sync + 'static,
        EFut: Future<Output = ERes> + Send,
        ERes: IntoResponse + Send,
    {
        let (mut parts, body) = req.into_parts();
        Box::pin(async move {
            let t1 = match T1::from_request_parts(&mut parts, &state).await {
                Ok(value) => value,
                Err(rejection) => return rejection.into_response(),
            };
            let t2 = match T2::from_request_parts(&mut parts, &state).await {
                Ok(value) => value,
                Err(rejection) => return rejection.into_response(),
            };

            let req = Request::from_parts(parts, body);

            match self(t1, t2).await {
                Ok(r) => r.into_response(),
                Err(err) => {
                    err_handler(req, err)
                        .await
                        .into_response()
                }
            }
        })
    }
}

impl<F, Fut, S, Res, T1, T2, T3> PmrHandlerExt<(T1, T2, T3), S> for F
where
    F: FnOnce(T1, T2, T3) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Res, AppError>> + Send,
    S: Send + Sync + 'static,
    Res: IntoResponse + Send,
    T1: FromRequestParts<S> + Send,
    T2: FromRequestParts<S> + Send,
    T3: FromRequestParts<S> + Send,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call<E, EFut, ERes>(self, req: Request, state: S, err_handler: E) -> Self::Future
    where
        E: FnOnce(Request, AppError) -> EFut + Clone + Send + Sync + 'static,
        EFut: Future<Output = ERes> + Send,
        ERes: IntoResponse + Send,
    {
        let (mut parts, body) = req.into_parts();
        Box::pin(async move {
            let t1 = match T1::from_request_parts(&mut parts, &state).await {
                Ok(value) => value,
                Err(rejection) => return rejection.into_response(),
            };
            let t2 = match T2::from_request_parts(&mut parts, &state).await {
                Ok(value) => value,
                Err(rejection) => return rejection.into_response(),
            };
            let t3 = match T3::from_request_parts(&mut parts, &state).await {
                Ok(value) => value,
                Err(rejection) => return rejection.into_response(),
            };

            let req = Request::from_parts(parts, body);

            match self(t1, t2, t3).await {
                Ok(r) => r.into_response(),
                Err(err) => {
                    err_handler(req, err)
                        .await
                        .into_response()
                }
            }
        })
    }
}

impl<F, Fut, S, Res, T1, T2, T3, T4> PmrHandlerExt<(T1, T2, T3, T4), S> for F
where
    F: FnOnce(T1, T2, T3, T4) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Res, AppError>> + Send,
    S: Send + Sync + 'static,
    Res: IntoResponse + Send,
    T1: FromRequestParts<S> + Send,
    T2: FromRequestParts<S> + Send,
    T3: FromRequestParts<S> + Send,
    T4: FromRequestParts<S> + Send,
{
    type Future = Pin<Box<dyn Future<Output = Response> + Send>>;

    fn call<E, EFut, ERes>(self, req: Request, state: S, err_handler: E) -> Self::Future
    where
        E: FnOnce(Request, AppError) -> EFut + Clone + Send + Sync + 'static,
        EFut: Future<Output = ERes> + Send,
        ERes: IntoResponse + Send,
    {
        let (mut parts, body) = req.into_parts();
        Box::pin(async move {
            let t1 = match T1::from_request_parts(&mut parts, &state).await {
                Ok(value) => value,
                Err(rejection) => return rejection.into_response(),
            };
            let t2 = match T2::from_request_parts(&mut parts, &state).await {
                Ok(value) => value,
                Err(rejection) => return rejection.into_response(),
            };
            let t3 = match T3::from_request_parts(&mut parts, &state).await {
                Ok(value) => value,
                Err(rejection) => return rejection.into_response(),
            };
            let t4 = match T4::from_request_parts(&mut parts, &state).await {
                Ok(value) => value,
                Err(rejection) => return rejection.into_response(),
            };

            let req = Request::from_parts(parts, body);

            match self(t1, t2, t3, t4).await {
                Ok(r) => r.into_response(),
                Err(err) => {
                    err_handler(req, err)
                        .await
                        .into_response()
                }
            }
        })
    }
}
