use leptos::{
    ServerFnError,
    server,
};
use pmrcore::{
    exposure::{
        Exposures,
    },
};
#[cfg(feature = "ssr")]
use crate::server::platform;

#[server]
pub async fn list_exposures() -> Result<Exposures, ServerFnError> {
    use pmrcore::exposure::traits::ExposureBackend;

    let platform = platform().await?;
    Ok(ExposureBackend::list(platform.mc_platform.as_ref())
        .await?)
}
