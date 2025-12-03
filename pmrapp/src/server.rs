use pmrctrl::platform::Platform;
use crate::{
    app::id::Id,
    error::AppError
};

pub async fn platform() -> Result<Platform, AppError> {
    Ok(leptos_axum::extract::<axum::Extension<Platform>>()
        .await
        .map_err(|_| AppError::InternalServerError)?
        .0
    )
}

pub fn log_error(error: impl std::fmt::Display) -> AppError {
    log::error!("{error}");
    AppError::InternalServerError
}

pub async fn resolve_id(kind: &'static str, id: Id) -> Result<i64, AppError> {
    let platform = platform().await?;
    id.resolve(&platform, kind).await
}

pub mod ac;
pub mod exposure;
pub mod workspace;
