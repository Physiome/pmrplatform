use pmrctrl::platform::Platform;
use crate::error::AppError;

pub async fn platform() -> Result<Platform, AppError> {
    Ok(leptos_axum::extract::<axum::Extension<Platform>>()
        .await
        .map_err(|_| AppError::InternalServerError)?
        .0
    )
}

pub mod workspace;
