use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("400 Bad Request")]
    BadRequest,
    #[error("401 Unauthorized")]
    Unauthorized,
    #[error("403 Forbidden")]
    Forbidden,
    #[error("404 Not Found")]
    NotFound,

    #[error("500 Internal Server Error")]
    Error,
    #[error("500 Internal Server Error")]
    SerdeJson(#[from] serde_json::error::Error),
    #[error("500 Internal Server Error")]
    Sqlx(#[from] sqlx::Error),
    #[error("500 Internal Server Error")]
    PmrRepoError(#[from] pmrrepo::error::PmrRepoError),
}


impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}


impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let body = match self {
            Self::PmrRepoError(ref e) => {
                log::info!("repo error: {:?}", e);
            },
            _ => (),
        };
        (self.status_code(), body).into_response()
    }
}
