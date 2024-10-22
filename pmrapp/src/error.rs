use http::status::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Serialize, Deserialize)]
pub enum AppError {
    #[error("403 Forbidden")]
    Forbidden,
    #[error("404 Not Found")]
    NotFound,

    #[error("500 Internal Server Error")]
    InternalServerError,
    #[error("500 Internal Server Error")]
    ViewNotImplemented,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[cfg(feature = "ssr")]
mod ssr {
    use axum::response::{IntoResponse, Response};
    use crate::error::AppError;

    impl IntoResponse for AppError {
        fn into_response(self) -> Response {
            // TODO bring in the standard template to wrap around this.
            let body = ();
            (self.status_code(), body).into_response()
        }
    }
}
