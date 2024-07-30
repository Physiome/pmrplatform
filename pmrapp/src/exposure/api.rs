use leptos::{
    prelude::ServerFnError,
    server,
};
use pmrcore::{
    exposure::{
        Exposures,
        ExposureFile,
    },
};
#[cfg(feature = "ssr")]
use crate::server::platform;

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
pub async fn get_file(
    id: i64, path: String,
) -> Result<Result<ExposureFile, String>, ServerFnError> {
    // TODO when there is a proper error type for id not found, use that
    use pmrcore::exposure::traits::Exposure as _;
    use pmrctrl::error::PlatformError::BackendError;

    use leptos_axum::ResponseOptions;
    use leptos::context::use_context;
    use http::StatusCode;

    let platform = platform().await?;
    let ec = platform.get_exposure(id).await?;
    match ec.ctrl_path(path.as_ref()).await {
        Ok(efc) => Ok(Ok(efc.exposure_file().clone_inner())),
        Err(error) => match error {
            BackendError(_) => {
                let exposure = ec.exposure();
                let path = format!(
                    "/workspace/{}/rawfile/{}/{}",
                    exposure.workspace_id(),
                    exposure.commit_id(),
                    path,
                );
                eprintln!("redir {path}");
                leptos_axum::redirect(path.as_ref());
                let res = use_context::<ResponseOptions>().unwrap();
                res.set_status(StatusCode::FOUND);
                dbg!(res.0);
                Ok(Err(path))
            },
            _ => Err(error.into()),
        }
    }
}
