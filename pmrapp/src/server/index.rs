use axum::{
    Extension,
    Json,
    extract::Path,
};
use pmrcore::index::{IndexTerms, IndexResourceDetailedSet};
use pmrctrl::platform::Platform;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

// Index listing

// Functions suffixed with `_core` are shared between Leptos server functions (at least until a way to unify
// server functions and the endpoint compatible with utoipa/OpenAPI specification.
pub(crate) async fn indexes_core(
    platform: &Platform,
) -> Result<Vec<String>, AppError> {
    platform.pc_platform.list_kinds().await
        .map_err(|_| AppError::InternalServerError)
}

// The struct here is named rather than newtype simply because these clearly denotes the
// mapping in both serde and also the OpenAPI specification (through utoipa which also uses
// serde) - this is specifically needed to wrap bare `Vec`s to prevent potential information
// leakage through `<script>` tags through `src` attribute.
#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Indexes {
    pub indexes: Vec<String>,
}

// The axum endpoint to be annotated with utoipa
#[cfg_attr(feature = "utoipa", utoipa::path(
    get,
    path = "/api/index",
    responses((
        status = 200,
        description = "Listing of available indexes",
        body = Indexes,
    ), AppError),
))]
pub async fn indexes(
    platform: Extension<Platform>,
) -> Result<Json<Indexes>, AppError> {
    let indexes = indexes_core(&platform.0).await?;
    Ok(Json(Indexes { indexes }))
}


// Index Terms
pub(crate) async fn terms_core(platform: &Platform, kind: String) -> Result<Option<IndexTerms>, AppError> {
    platform.pc_platform.list_terms(&kind).await
        .map_err(|_| AppError::InternalServerError)
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    get,
    path = "/api/index/{kind}", params(
        ("kind" = String, Path, description = "The kind of index to load."),
    ),
    responses((
        status = 200,
        description = "Listing of terms for the kind of index.",
        body = Option<IndexTerms>,
    ), AppError),
))]
pub async fn terms(
    platform: Extension<Platform>,
    Path(kind): Path<String>,
) -> Result<Json<Option<IndexTerms>>, AppError> {
    Ok(Json(terms_core(&platform.0, kind).await?))
}


// Index Resource Set
pub(crate) async fn resources_core(
    platform: &Platform,
    kind: String,
    term: String,
) -> Result<Option<IndexResourceDetailedSet>, AppError> {
    platform.pc_platform.list_resources_details(&kind, &term).await
        .map_err(|_| AppError::InternalServerError)
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    get,
    path = "/api/index/{kind}/{term}",
    params(
        ("kind" = String, Path, description = "The `kind` of index."),
        ("term" = String, Path, description = "Term to load."),
    ),
    responses((
        status = 200,
        description = "Listing of resources by the term under a kind from the index.",
        body = Option<IndexResourceDetailedSet>,
    ), AppError),
))]
pub async fn resources(
    platform: Extension<Platform>,
    Path((kind, term)): Path<(String, String)>,
) -> Result<Json<Option<IndexResourceDetailedSet>>, AppError> {
    Ok(Json(resources_core(&platform.0, kind, term).await?))
}
