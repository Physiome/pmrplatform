use thiserror::Error;

pub mod task;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BackendError {
    #[cfg(feature = "sqlx")]
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    /// Denotes custom application invariant; generally informative.
    #[error("application invariant violated: {0}")]
    AppInvariantViolation(String),
    #[error("unknown error")]
    Unknown,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ValueError {
    #[error(transparent)]
    Backend(#[from] BackendError),
    #[error("uninitialized value")]
    Uninitialized,
}
