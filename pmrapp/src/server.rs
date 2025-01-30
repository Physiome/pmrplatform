use pmrctrl::platform::Platform;
use crate::error::AppError;

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

pub mod ac;
pub mod exposure;
pub mod workspace;
