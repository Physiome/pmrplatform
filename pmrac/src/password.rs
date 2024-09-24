use argon2::password_hash::PasswordHash;

/// These represent special plain-text "passwords" with special meaning.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Password<'a> {
    Misconfigured,
    New,
    Reset,
    Restricted,
    Hash(PasswordHash<'a>),
    Raw(&'a str),
}

mod impls;
