use crate::{
    app::id::Id,
    error::AppError,
};

/// Helper enum for declaring the current root and mode
#[derive(Clone, Copy)]
pub enum Root {
    /// Root instantiated with this mode declares the route to allow alias for use with params.
    Aliased(&'static str),
    /// Root instantiated with this mode declares the route to be reference the id of the underlying type.
    Id(&'static str),
}

mod display {
    use std::fmt::{Display, Formatter, Result};
    use super::*;

    impl Display for Root {
        fn fmt(&self, f: &mut Formatter) -> Result {
            match self {
                Self::Aliased(s) => s.fmt(f),
                Self::Id(s) => s.fmt(f),
            }
        }
    }
}

impl AsRef<str> for Root {
    fn as_ref(&self) -> &'static str {
        match self {
            Self::Aliased(s) => s,
            Self::Id(s) => s,
        }
    }
}

impl Root {
    pub fn build_id(&self, s: String) -> Result<Id, AppError> {
        Ok(match self {
            Root::Aliased(_) => Id::Aliased(s),
            Root::Id(_) => Id::Number(s.parse().map_err(|_| AppError::NotFound)?),
        })
    }

    pub fn build_href(&self, s: String) -> String {
        match self {
            Root::Aliased(prefix) => format!("{prefix}{s}"),
            Root::Id(prefix) => format!("{prefix}{s}"),
        }
    }
}
