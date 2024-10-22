use leptos::prelude::ServerFnError;
use http::status::StatusCode;
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    str::FromStr,
};
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
            AppError::Forbidden => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl FromStr for AppError {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // This is converting the output of the Display impl by thiserror
        Ok(match s {
            "403 Forbidden" => AppError::Forbidden,
            "404 Not Found" => AppError::NotFound,
            // anything else is considered an InternalServerError
            _ => AppError::InternalServerError,
        })
    }
}

impl From<ServerFnError<AppError>> for AppError {
    fn from(e: ServerFnError<AppError>) -> Self {
        match e {
            ServerFnError::WrappedServerError(e) => e,
            _ => Self::InternalServerError,
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
