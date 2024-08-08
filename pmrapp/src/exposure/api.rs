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
) -> Result<Result<(ExposureFile, ExposureFileView), AppError>, ServerFnError> {
    // TODO when there is a proper error type for id not found, use that
    use pmrcore::exposure::traits::Exposure as _;
    use pmrctrl::error::CtrlError;

    let platform = platform().await?;
    let ec = platform.get_exposure(id).await?;

    match ec.resolve_file_view(path.as_ref()).await {
        Ok(Ok(efvc)) => Ok(Ok((
            efvc.exposure_file_ctrl().exposure_file().clone_inner(),
            efvc.exposure_file_view().clone_inner()
        ))),
        Ok(Err(_)) | Err(CtrlError::EFCNotFound(_)) => {
            // since the request path has a direct hit on file, doesn't
            // matter if ExposureFileCtrl found or not.
            let exposure = ec.exposure();
            let path = format!(
                "/workspace/{}/rawfile/{}/{}",
                exposure.workspace_id(),
                exposure.commit_id(),
                path,
            );
            // leptos_axum::redirect(path.as_str());
            Ok(Err(AppError::Redirect(path).into()))
        },
        // CtrlError::UnknownPath(_) | CtrlError::EFVCNotFound(_)
        _ => Err(AppError::NotFound.into()),
    }
}
