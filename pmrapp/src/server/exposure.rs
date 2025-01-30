use axum::{
    Extension,
    response::IntoResponse,
};
use axum_login::AuthSession;
use pmrac::Platform as ACPlatform;
use pmrctrl::platform::Platform;
use crate::{
    error::AppError,
    server::platform,
};


pub async fn wizard_field_update(
    platform: Extension<Platform>,
    session: Extension<AuthSession<ACPlatform>>,
    body: String,
) -> Result<impl IntoResponse, AppError> {
    // parse all the keys for the identifiers
    // get the identifier that points out to the exposure that the fields are restricted to
    // - this is to avoid unchecked bulk update of arbitrary fields
    // - only update fields that are for the exposure.
    let values = serde_urlencoded::from_str::<Vec<(String, String)>>(&body)
        .map_err(|_| AppError::InternalServerError)?;
    dbg!(values);
    Ok("")
}
