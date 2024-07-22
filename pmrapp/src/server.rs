use leptos::{
    ServerFnError,
};
use pmrctrl::platform::Platform;

pub async fn platform() -> Result<Platform, ServerFnError> {
    Ok(leptos_axum::extract::<axum::Extension<Platform>>()
        .await?
        .0
    )
}

pub mod workspace;
