use leptos::server;
use pmrcore::citation::Citation;

use crate::error::AppError;

#[cfg(feature = "ssr")]
mod ssr {
    pub use crate::server::platform;
}
#[cfg(feature = "ssr")]
use self::ssr::*;

#[server]
pub async fn list_citations() -> Result<Vec<Citation>, AppError> {
    let platform = platform().await?;
    platform.pc_platform.list_citations().await
        .map_err(|_| AppError::InternalServerError)
}

#[server]
pub async fn list_citation_resources(identifier: String) -> Result<Vec<String>, AppError> {
    let platform = platform().await?;
    platform.pc_platform.list_citation_resources(&identifier).await
        .map_err(|_| AppError::InternalServerError)
}
