use thiserror::Error;

pub mod task;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Backend(#[from] BackendError),
    #[error(transparent)]
    Value(#[from] ValueError),
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum BackendError {
    #[cfg(feature = "sqlx")]
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    /// Denotes custom application invariant; generally informative.
    #[error("application invariant violated: {0}")]
    AppInvariantViolation(String),
    #[error("cannot bind an entity to a non-matching backend")]
    NonMatchingBind,
    #[error("unknown error")]
    Unknown,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ValueError {
    #[error("entity missing: {0}")]
    EntityMissing(String),
    #[error(transparent)]
    Task(#[from] task::TaskError),
    #[error("uninitialized value")]
    Uninitialized,
    #[error("uninitialized attribute: {0}")]
    UninitializedAttribute(&'static str),
}
