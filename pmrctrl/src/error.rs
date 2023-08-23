use pmrcore::error::BackendError;
use pmrrepo::error::PmrRepoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error(transparent)]
    BackendError(#[from] BackendError),
    #[error(transparent)]
    PmrRepoError(#[from] PmrRepoError),
}
