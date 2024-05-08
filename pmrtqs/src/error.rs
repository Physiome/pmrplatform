use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum RunnerError {
    // TODO figure out how much info to actually include here
    #[error(transparent)]
    Backend(#[from] pmrcore::error::BackendError),
    #[error("non-zero exit error for task_id: {0} (code: {1})")]
    NonZero(i64, i64),
    #[error(transparent)]
    Stdio(#[from] std::io::Error),
    #[error(transparent)]
    ValueError(#[from] pmrcore::error::ValueError),
}
