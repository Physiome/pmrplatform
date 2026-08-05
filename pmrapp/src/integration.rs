use axum::{
    extract::{
        Extension,
        Request,
    },
    http::{
        Uri,
        header::{self, HeaderValue},
        uri::{
            Authority,
            PathAndQuery,
            Scheme,
        },
    },
    routing::{
        get,
        post,
    },
};
use http::Method;
use axum_login_bearer::BearerTokenAuthManagerLayer;
use pmrcore::web::Source;
use pmrctrl::platform::Platform;
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
pub trait PmrAxumExt: private::Sealed {
    /// Set up the routes related to PMR.
    ///
    /// Please ensure that `.layers()` is called at some point after PMR routes have been added.
    fn pmr_routes(self) -> Self;

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

impl<S> PmrAxumExt for axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn pmr_routes(self) -> Self {
        let app = self
            .without_v07_checks()
            // TODO the path should be constructed from a known list, so that rewriting only happens
            // to this route only if it exists.
            .route("/api/exposure/{e_id}/{ef_id}/{view_key}/{*path}", get(exposure_file_data))
            .route("/api/exposure/safe_html/{e_id}/{ef_id}/{view_key}/{*path}", get(exposure_file_safe_html))
            .route("/collection_json/workspace/", get(collection_json_workspace))

            .route("/api/exposure/{exposure_id}/download_zip", get(aliased_exposure_archive_zip))
            .route("/exposure/{exposure_id}/download_zip", get(aliased_exposure_archive_zip))

            // These are duplicated to /api/ to keep the OpenAPI specification consistent, while
            // keeping the original in the event we will fall back to a fully integrated application.
            .route("/workspace/{workspace_alias}/archive/{commit_id}/tgz", get(aliased_workspace_archive_tgz))
            .route("/workspace/:/id/{workspace_id}/archive/{commit_id}/tgz", get(workspace_archive_tgz))
            .route("/workspace/{workspace_alias}/archive/{commit_id}/zip", get(aliased_workspace_archive_zip))
            .route("/workspace/:/id/{workspace_id}/archive/{commit_id}/zip", get(workspace_archive_zip))
            .route("/workspace/{workspace_alias}/rawfile/{commit_id}/{*path}", get(aliased_workspace_rawfile_download))
            .route("/workspace/:/id/{workspace_id}/rawfile/{commit_id}/{*path}", get(workspace_rawfile_download))
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
        let app = app.merge(
            utoipa_swagger_ui::SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", crate::openapi::ApiDoc::openapi())
        );

        app
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
