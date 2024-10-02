use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash,
        PasswordHasher,
        PasswordVerifier,
        SaltString,
    },
    Argon2,
};
use std::fmt;

use crate::error::PasswordError;
use super::{
    Password,
    PasswordStatus,
};

impl fmt::Display for Password<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<Password<'_>> for PasswordStatus {
    fn from(val: Password<'_>) -> Self {
        match val {
            Password::Misconfigured => PasswordStatus::Misconfigured,
            Password::New => PasswordStatus::New,
            Password::Reset => PasswordStatus::Reset,
            Password::Restricted => PasswordStatus::Restricted,
            Password::Hash(_) => PasswordStatus::Hash,
            Password::Raw(_) => PasswordStatus::Raw,
        }
    }
}

impl<'a> Password<'a> {
    pub fn new(s: &'a str) -> Self {
        Password::Raw(s)
    }

    pub fn from_database(s: &'a str) -> Self {
        match s {
            "New" => Password::New,
            "Reset" => Password::Reset,
            "Restricted" => Password::Restricted,
            _ => PasswordHash::new(s)
                .map(Password::Hash)
                .unwrap_or(Password::Misconfigured)
        }
    }

    pub fn to_database(self) -> Result<String, PasswordError> {
        match self {
            Password::Hash(h) => Ok(h.to_string()),
            Password::Raw(s) => {
                let salt = SaltString::generate(&mut OsRng);
                Ok(Argon2::default()
                    .hash_password(s.as_bytes(), &salt)?
                    .to_string()
                )
            },
            p => Ok(format!("{p}")),
        }
    }

    pub fn verify(&'a self, other: &Self) -> Result<(), PasswordError> {
        match (self, other) {
            (Password::Hash(hash), Password::Raw(raw)) |
            (Password::Raw(raw), Password::Hash(hash)) => {
                Argon2::default()
                    .verify_password(raw.as_bytes(), hash)
                    .map_err(|_| PasswordError::Wrong)
            },
            _ => Err(PasswordError::NotVerifiable)?,
        }
    }
}
