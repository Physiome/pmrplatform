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
    Ok(match id {
        Id::Number(s) => s.parse().map_err(|_| AppError::NotFound)?,
        Id::Aliased(s) => platform()
            .await?
            .mc_platform
            .resolve_alias(kind, &s)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or(AppError::NotFound)?,
    })
}

pub mod ac;
pub mod exposure;
pub mod workspace;
