use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("task already has already been queued with id: {0}")]
    TaskAlreadyQueued(i64),
    #[error(transparent)]
    BackendError(#[from] crate::error::BackendError),
}
