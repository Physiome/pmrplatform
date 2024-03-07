use pmrcore::error::{
    BackendError,
    ValueError,
    task::TaskError,
};
use pmrmodel::error::BuildArgErrors;
use pmrrepo::error::PmrRepoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error(transparent)]
    BackendError(#[from] BackendError),
    #[error(transparent)]
    BuildArgErrors(#[from] BuildArgErrors),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    PmrRepoError(#[from] PmrRepoError),
    #[error(transparent)]
    TaskError(#[from] TaskError),
    #[error(transparent)]
    ValueError(#[from] ValueError),
}
