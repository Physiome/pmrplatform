use std::fmt::{Display, Formatter, Result};

/// Helper enum for declaring the current root and mode
#[derive(Clone, Copy)]
pub enum Root {
    /// Root instantiated with this mode declares the route to allow alias for use with params.
    Aliased(&'static str),
    /// Root instantiated with this mode declares the route to be reference the id of the underlying type.
    Id(&'static str),
}

impl Display for Root {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::Aliased(s) => s.fmt(f),
            Self::Id(s) => s.fmt(f),
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

// TODO ideally, we need a trait for resolving real items
// they need to be implemented for workspace and exposure records
