use thiserror::Error;

pub mod task;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BackendError {
    #[cfg(feature = "sqlx")]
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error("unknown error")]
    Unknown,
}
