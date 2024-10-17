use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Backend(#[from] pmrcore::error::BackendError),
    #[error(transparent)]
    Password(#[from] PasswordError),
    #[error(transparent)]
    Authentication(#[from] AuthenticationError),
    #[error("Misconfiguration Password")]
    Misconfiguration,
    #[error(transparent)]
    Rbac(#[from] pmrrbac::error::Error),
}

#[non_exhaustive]
#[derive(Debug, Error, PartialEq)]
pub enum PasswordError {
    #[error(transparent)]
    Argon2(#[from] argon2::password_hash::Error),
    #[error("Existing Password")]
    Existing,
    #[error("Mismatched Password")]
    Mismatched,
    #[error("Wrong Password")]
    Wrong,
    #[error("Not Verifiable")]
    NotVerifiable,
}

#[non_exhaustive]
#[derive(Debug, Error, PartialEq)]
pub enum AuthenticationError {
    #[error(transparent)]
    Password(#[from] PasswordError),
    #[error("Restricted")]
    Restricted,
    #[error("UnknownUser")]
    UnknownUser,
}
