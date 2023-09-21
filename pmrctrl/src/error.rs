use pmrcore::error::{
    BackendError,
    ValueError,
    task::TaskError,
};
use pmrrepo::error::PmrRepoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error(transparent)]
    BackendError(#[from] BackendError),
    #[error(transparent)]
    PmrRepoError(#[from] PmrRepoError),
    #[error(transparent)]
    TaskError(#[from] TaskError),
    #[error(transparent)]
    ValueError(#[from] ValueError),
}
