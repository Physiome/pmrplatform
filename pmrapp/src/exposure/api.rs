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
    use pmrmodel::backend::db::SqliteBackend;
    use pmrctrl::platform::Platform;
    let platform = leptos_axum::extract::<axum::Extension<Platform<SqliteBackend, SqliteBackend>>>()
        .await?
        .0;
    Ok(ExposureBackend::list(platform.mc_platform.as_ref())
        .await?)
}
