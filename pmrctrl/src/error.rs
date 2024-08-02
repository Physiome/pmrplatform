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
    CtrlError(#[from] CtrlError),
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

#[derive(Debug, PartialEq, Error)]
pub enum CtrlError {
    /// Path that isn't known under the associated resource
    #[error("unknown path: {0}")]
    UnknownPath(String),
    /// Path is valid, but is not associated with a ExposureFileCtrl
    #[error("exposure file not found: {0}")]
    EFCNotFound(String),
    // TODO how to disambiguate a path that shares a known path, but it
    // might or might not have a ExposureFile which might or might not
    // have a ExposureFileView
}
