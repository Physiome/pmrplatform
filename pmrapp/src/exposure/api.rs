use leptos::{
    prelude::ServerFnError,
    server,
};
use pmrcore::{
    exposure::{
        Exposures,
        ExposureFile,
        ExposureFileView,
    },
};
#[cfg(feature = "ssr")]
use crate::server::platform;

use crate::error::AppError;

#[server]
pub async fn list() -> Result<Exposures, ServerFnError> {
    use pmrcore::exposure::traits::ExposureBackend;

    let platform = platform().await?;
    Ok(ExposureBackend::list(platform.mc_platform.as_ref())
        .await?)
}

#[server]
pub async fn list_files(id: i64) -> Result<Vec<(String, bool)>, ServerFnError> {
    let platform = platform().await?;
    let ctrl = platform.get_exposure(id).await?;
    Ok(ctrl.list_files_info().await?)
}

#[server]
pub async fn resolve_exposure_path(
    id: i64,
    path: String,
) -> Result<Result<(ExposureFile, Result<ExposureFileView, Vec<String>>), AppError>, ServerFnError> {
    // TODO when there is a proper error type for id not found, use that
    // TODO ExposureFileView is a placeholder - the real type that should be returned
    // is something that can readily be turned into an IntoView.

    // Currently, if no underlying errors that may result in a
    // ServerFnError is returned, a Result::Ok that contains an
    // additional Result which either is a redirect to some target
    // resource (be it the underlying path to some workspace, or some
    // specific default view), or that inner Ok will containing a
    // 2-tuple with the first element being the ExposureFile and the
    // second being a result that would be the ExposureFileView or a
    // vec of strings of valid view_keys that could be navigated to.

    use pmrcore::exposure::traits::{
        Exposure as _,
        ExposureFile as _,
        ExposureFileView as _,
    };
    use pmrctrl::error::CtrlError;

    let platform = platform().await?;
    let ec = platform.get_exposure(id).await?;

    match ec.resolve_file_view(path.as_ref()).await {
        (Ok(efc), Ok(efvc)) => Ok(Ok((
            efc.exposure_file().clone_inner(),
            Ok(efvc.exposure_file_view().clone_inner()),
        ))),
        (_, Err(CtrlError::None)) => {
            // since the request path has a direct hit on file, doesn't
            // matter if ExposureFileCtrl found or not.
            let exposure = ec.exposure();
            let path = format!(
                "/workspace/{}/rawfile/{}/{}",
                exposure.workspace_id(),
                exposure.commit_id(),
                path,
            );
            // Not using the integration as it's not currently set up properly yet
            // (the redirect hooks are missing for server functions for the routes)
            // leptos_axum::redirect(path.as_str());
            // returning this as an Ok(Err(..)) to avoid redirecting while this is a response for csr
            Ok(Err(AppError::Redirect(path).into()))
        },
        (Ok(efc), Err(CtrlError::EFVCNotFound(viewstr))) if viewstr == "" => Ok(Ok((
            efc.exposure_file().clone_inner(),
            Err(efc.exposure_file()
                .views()
                .await?
                .iter()
                .filter_map(|v| v.view_key().map(str::to_string))
                .collect::<Vec<_>>()
            ),
        ))),
        // CtrlError::UnknownPath(_) | CtrlError::EFVCNotFound(_)
        _ => Err(AppError::NotFound.into()),
    }
}
