use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Argon2(#[from] argon2::password_hash::Error),
    #[error(transparent)]
    Backend(#[from] pmrcore::error::BackendError),
    #[error(transparent)]
    PasswordError(#[from] PasswordError),
}

#[non_exhaustive]
#[derive(Debug, Error, PartialEq)]
pub enum PasswordError {
    #[error("Mismatched Password")]
    MismatchedPassword,
    #[error("Wrong Password")]
    WrongPassword,
}
