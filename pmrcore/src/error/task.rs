use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("task {0} is stale as it is no longer referenced as needed")]
    StaleNoRef(i64),
    #[error("task already has already been queued with id: {0}")]
    TaskAlreadyQueued(i64),
    #[error(transparent)]
    BackendError(#[from] crate::error::BackendError),
}
