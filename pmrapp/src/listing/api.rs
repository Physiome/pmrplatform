use leptos::server;
use pmrcore::{
    citation::Citation,
    index::{IndexTerms, IndexResourceDetailedSet},
};

use crate::error::AppError;

#[cfg(feature = "ssr")]
mod ssr {
    pub use crate::server::platform;
    pub use crate::server::index;
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
pub async fn list_indexes() -> Result<Vec<String>, AppError> {
    let platform = platform().await?;
    index::indexes_core(&platform).await
}

#[server]
pub async fn list_index_terms(kind: String) -> Result<Option<IndexTerms>, AppError> {
    let platform = platform().await?;
    index::terms_core(&platform, kind).await
}

#[server]
pub async fn list_indexed_resources_by_kind_term(
    kind: String,
    term: String,
) -> Result<Option<IndexResourceDetailedSet>, AppError> {
    let platform = platform().await?;
    index::resources_core(&platform, kind, term).await
}
