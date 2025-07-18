use crate::{
    app::id::Id,
    error::AppError,
};

/// Helper enum for declaring the current root and mode
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Root {
    /// Root instantiated with this mode declares the route to allow alias for use with params.
    Aliased(&'static str),
    /// Root instantiated with this mode declares the route to be reference the id of the underlying type.
    Id(&'static str),
}

/// A defined root for some singular entity with an single identifier.
#[derive(Clone, Eq, PartialEq)]
pub struct EntityRoot {
    root: Root,
    id: String,
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

    pub fn build_entity_root(&self, id: String) -> EntityRoot {
        EntityRoot {
            root: *self,
            id
        }
    }
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

    impl Display for EntityRoot {
        fn fmt(&self, f: &mut Formatter) -> Result {
            write!(f, "{}{}", self.root, self.id)
        }
    }
}
