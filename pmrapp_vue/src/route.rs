use axum::{
    Extension,
    body::Body,
    extract::{
        FromRequestParts,
        Path,
    },
    http::{
        Request,
        StatusCode,
    },
    response::{
        IntoResponse,
        Redirect,
        Response,
    },
    routing::get,
};
use axum_login::AuthSession;
use pmrac::Platform as ACPlatform;
use pmrapp::{
    error::AppError,
    server::ac::Session,
};
use pmrcore::exposure::traits::Exposure as _;
use pmrctrl::{
    error::CtrlError,
    platform::Platform,
};
use tower::util::ServiceExt;
use tower_http::services::ServeFile;

use crate::service::ConditionalServeFile;

const VUE_APP_ROUTES: &[&str] = &[
    // Existing PMR2 and Leptos frontend.
    "/workspace/",
    "/workspace",
    "/workspace/{id}/",
    "/workspace/{id}",
    "/workspace/{id}/synchronize",
    // FIXME This need to be handled specifically for 404.
    "/workspace/{id}/file/{commit}/{*path}",
    "/workspace/{id}/create_exposure/{commit}",
    "/workspace/{id}/log",

    "/exposure/",
    "/exposure",
    "/exposure/{id}/",
    "/exposure/{id}",
    "/exposure/{id}/:/wizard",
    // FIXME This should be handled specifically for redirects and 404.
    // "/exposure/{id}/{*path}",

    // Current Vue frontend also support these additional paths which are the plural form.
    "/workspaces/",
    "/workspaces",
    "/workspaces/{id}/",
    "/workspaces/{id}",
    "/workspaces/{id}/synchronize",
    // FIXME This need to be handled specifically for 404.
    "/workspaces/{id}/file/{commit}/{*path}",
    "/workspaces/{id}/create_exposure/{commit}",
    "/workspaces/{id}/log",

    "/exposures/",
    "/exposures",
    "/exposures/{id}/",
    "/exposures/{id}",
    "/exposures/{id}/:/wizard",
    // FIXME This should be handled specificall for redirects and 404.
    // "/exposures/{id}/{*path}",

    // Not currently implemented (or necessary), but was provided with Leptos frontend.
    // "/workspace/:/add",
    // "/workspace/:/id/",
    // "/workspace/:/id/{id}/",
    // "/workspace/:/id/{id}",
    // "/workspace/:/id/{id}/synchronize",
    // "/workspace/:/id/{id}/file/{commit}/{*path}",
    // "/workspace/:/id/{id}/create_exposure/{commit}",
    // "/workspace/:/id/{id}/log",

    // "/exposure/:/id/",
    // "/exposure/:/id/{id}/",
    // "/exposure/:/id/{id}",
    // "/exposure/:/id/{id}/:/wizard",
    // "/exposure/:/id/{id}/{*path}",

    // Raw listing done on the Leptos frontend, may be required later.
    // "/catalog/",
    // "/catalog/{kind}/",
    // "/catalog/{kind}/{term}",
    // "/listing/",
    // "/listing/by-reference/",
    // "/listing/by-reference/{id}",

    // "/auth/",
    // "/auth/login",
    // "/auth/logged_out",

    // Additional paths implemented in the Vue frontend that need a proper response.
    "/search",
    "/search/",
    "/login",
    "/login/",
    "/profile",
    "/profile/",
];


mod private {
    pub trait Sealed {}
    impl<S> Sealed for axum::Router<S> {}
}

/// Extension trait for [`axum::Router`] so it may be set up for `pmrapp-frontend` Vue application routes.
pub trait PmrVueAxumExt: private::Sealed {
    /// Set up routes required by the `pmrapp-frontend` Vue package.
    ///
    /// The asset path to the built `pmrapp-frontend` distribution must be provided.
    fn pmr_vue_routes(self, asset_path: &std::path::Path) -> Self;
}

impl<S> PmrVueAxumExt for axum::Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn pmr_vue_routes(self, asset_path: &std::path::Path) -> Self {
        let mut router = self.without_v07_checks();
        let serve_file = ServeFile::new(asset_path.join("index.html"));
        for path in VUE_APP_ROUTES.iter() {
            router = router.route_service(path, serve_file.clone());
        }
        let exposure_file_service = ConditionalServeFile::new(
            exposure_file_handler,
            serve_file,
        );

        router
            .route_service("/exposure/{id}/{*path}", exposure_file_service.clone())
            .route_service("/exposures/{id}/{*path}", exposure_file_service.clone())
    }
}

async fn exposure_file_handler(req: Request<Body>) -> Result<Option<Response>, StatusCode> {
    let platform = req.extensions().get::<Platform>()
        .expect("platform should have been provided as an extension")
        .clone();
    let session = req.extensions().get::<AuthSession<ACPlatform>>()
        .expect("session should have been provided as an extension")
        .clone();
    let (mut parts, _body) = req.into_parts();
    let Path((exposure_alias, path)) = Path::<(String, String)>::from_request_parts(&mut parts, &())
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let exposure_id = platform
        .mc_platform()
        .resolve_alias("exposure", &exposure_alias)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Session::from(session)
        .enforcer(format!("/exposure/{exposure_id}/"), "").await
        // FIXME the correct status code should be handled from AppError
        .map_err(|_| StatusCode::FORBIDDEN)?;

    // TODO When there is a proper error type for id not found, ensure NotFound is returned.
    let ec = platform.get_exposure(exposure_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let dummy = String::new();

    match (ec.resolve_file_view(path.as_ref()).await, dummy) {
        ((_, Err(CtrlError::None)), _) => {
            // Request path has a direct hit on some file, generate the appropriate redirect.
            let exposure = ec.exposure();
            let path = platform.get_workspace(exposure.workspace_id()).await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .alias()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .map_or_else(
                    || format!(
                        "/workspace/:/id/{}/rawfile/{}/{path}",
                        exposure.workspace_id(),
                        exposure.commit_id(),
                    ),
                    |alias| format!(
                        "/workspace/{alias}/rawfile/{}/{path}",
                        exposure.commit_id(),
                    ),
                );
            Ok(Some(Redirect::temporary(&path).into_response()))
        },
        ((Ok(_), Ok(_)), viewstr) |
        ((Ok(_), Err(CtrlError::EFVCNotFound(viewstr))), _) if viewstr == "" => {
            // Return the `index.html`.
            Ok(None)
        }
        // CtrlError::UnknownPath(_) | CtrlError::EFVCNotFound(_)
        _ => Err(StatusCode::NOT_FOUND),
    }
}
