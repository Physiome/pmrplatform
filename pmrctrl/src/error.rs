use pmrcore::error::{
    BackendError,
    Error,
    ValueError,
    task::TaskError,
};
use pmrmodel::error::BuildArgErrors;
use pmrrepo::error::PmrRepoError;
use pmrtqs::error::RunnerError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error(transparent)]
    BackendError(#[from] BackendError),
    #[error(transparent)]
    BuildArgErrors(#[from] BuildArgErrors),
    // FIXME BackendError and ValueError may need to be merged?
    #[error(transparent)]
    CoreError(#[from] Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    PmrRepoError(#[from] PmrRepoError),
    #[error(transparent)]
    RunnerError(#[from] RunnerError),
    #[error(transparent)]
    TaskError(#[from] TaskError),
    #[error(transparent)]
    ValueError(#[from] ValueError),
}
