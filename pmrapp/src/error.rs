use leptos::server_fn::{
    codec::JsonEncoding,
    error::{
        FromServerFnError,
        ServerFnErrorErr,
        ServerFnError,
    },
};
use http::status::StatusCode;
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    fmt,
    str::FromStr,
};
use thiserror::Error;

#[cfg_attr(feature = "utoipa", derive(utoipa::IntoResponses))]
#[derive(Clone, Debug, Error, PartialEq, Serialize, Deserialize)]
pub enum AppError {
    #[cfg_attr(feature = "utoipa", response(
        status = 400,
        description = "Bad Request",
    ))]
    #[error("400 Bad Request")]
    BadRequest,
    #[cfg_attr(feature = "utoipa", response(
        status = 403,
        description = "Forbidden",
    ))]
    #[error("403 Forbidden")]
    Forbidden,
    #[cfg_attr(feature = "utoipa", response(
        status = 404,
        description = "Not Found",
    ))]
    #[error("404 Not Found")]
    NotFound,

    #[cfg_attr(feature = "utoipa", response(
        status = 500,
        description = "Internal Server Error",
    ))]
    #[error("500 Internal Server Error")]
    InternalServerError,
    #[cfg_attr(feature = "utoipa", response(
        status = 500,
        description = "Internal Server Error",
    ))]
    #[error("500 Internal Server Error")]
    ViewNotImplemented,

    // other non-http error
    #[cfg_attr(feature = "utoipa", response(
        status = 500,
        description = "Internal Server Error",
    ))]
    #[error("Network Error")]
    NetworkError,
    #[cfg_attr(feature = "utoipa", response(
        status = 500,
        description = "Internal Server Error",
    ))]
    #[error("Encode/decode error")]
    SerdeError,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest => StatusCode::BAD_REQUEST,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<StatusCode> for AppError {
    // Only convert into status known to AppError
    fn from(value: StatusCode) -> Self {
        match value {
            StatusCode::BAD_REQUEST => AppError::BadRequest,
            StatusCode::NOT_FOUND => AppError::NotFound,
            StatusCode::FORBIDDEN => AppError::Forbidden,
            _ => AppError::InternalServerError,
        }
    }
}

impl FromStr for AppError {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // This is converting the output of the Display impl by thiserror
        Ok(match s {
            "400 Bad Request" => AppError::BadRequest,
            "403 Forbidden" => AppError::Forbidden,
            "404 Not Found" => AppError::NotFound,
            // anything else is considered an InternalServerError
            _ => AppError::InternalServerError,
        })
    }
}

impl From<ServerFnError> for AppError {
    fn from(e: ServerFnError) -> Self {
        match e {
            ServerFnError::Request(_) => Self::NetworkError,
            ServerFnError::Deserialization(_) |
            ServerFnError::Serialization(_) => Self::SerdeError,
            _ => Self::InternalServerError,
        }
    }
}

impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(e: ServerFnErrorErr) -> Self {
        match e {
            ServerFnErrorErr::Request(_) => Self::NetworkError,
            ServerFnErrorErr::Deserialization(_) |
            ServerFnErrorErr::Serialization(_) => Self::SerdeError,
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum AuthError {
    InternalServerError,
    InvalidCredentials,
    NetworkError,
    SerdeError,
}

impl From<AuthError> for &'static str {
    fn from(v: AuthError) -> &'static str {
        match v {
            AuthError::InternalServerError => "Internal server error",
            AuthError::InvalidCredentials => "Invalid credentials provided",
            AuthError::NetworkError => "Network error",
            AuthError::SerdeError => "Encoding error (is the application out of date?)",
        }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", <&'static str>::from(*self))
    }
}

impl FromServerFnError for AuthError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(e: ServerFnErrorErr) -> Self {
        match e {
            ServerFnErrorErr::Request(_) => Self::NetworkError,
            ServerFnErrorErr::Deserialization(_) |
            ServerFnErrorErr::Serialization(_) => Self::SerdeError,
            _ => Self::InternalServerError,
        }
    }
}
