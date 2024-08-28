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
use leptos::server_fn::codec::Rkyv;
use crate::error::AppError;

#[cfg(feature = "ssr")]
mod ssr {
    pub use crate::server::platform;
    pub use pmrcore::exposure::traits::ExposureBackend;
    pub use pmrcore::exposure::traits::{
        Exposure as _,
        ExposureFile as _,
        ExposureFileView as _,
    };
    pub use pmrctrl::error::CtrlError;
}
#[cfg(feature = "ssr")]
use self::ssr::*;

#[server]
pub async fn list() -> Result<Exposures, ServerFnError> {
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
) -> Result<Result<(ExposureFile, Result<(ExposureFileView, Option<String>), Vec<String>>), AppError>, ServerFnError> {
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

    let platform = platform().await?;
    let ec = platform.get_exposure(id).await?;

    match ec.resolve_file_view(path.as_ref()).await {
        (Ok(efc), Ok(efvc)) => Ok(Ok((
            efc.exposure_file().clone_inner(),
            Ok((
                efvc.exposure_file_view().clone_inner(),
                efvc.view_path().map(str::to_string),
            )),
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
            // Not using this built-in axum integration.
            // leptos_axum::redirect(path.as_str());
            //
            // Reason for doing our own thing here is because the target
            // is an endpoint outside of the leptos application and thus
            // not routable/renderable using CSR.
            //
            // Returning the path as an Ok(Err(..)) to indicate a proper
            // result that isn't a ServerFnError, but still an inner Err
            // to facilitate this custom redirect handling.
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

// TODO this should NOT be a full-on server function, but should be something
// that is only available on the server side, with dedicated client functions
// for each specific response type.  This is only acceptable in a prototype.
// FIXME remove server macro
#[server(ReadBlob, "/api", output = Rkyv)]
pub async fn read_blob(
    id: i64,
    path: String,
    efvid: i64,
    key: String,
) -> Result<Box<[u8]>, ServerFnError> {
    let platform = platform().await?;
    let ec = platform.get_exposure(id).await?;
    let efc = ec.ctrl_path(&path).await?;
    let efvc = efc.get_view(efvid).await?;
    Ok(efvc.read_blob(&key).await?)
}
