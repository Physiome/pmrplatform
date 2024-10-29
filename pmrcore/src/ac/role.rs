use enumset::{EnumSet, EnumSetType};
use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Debug, Default, EnumSetType, Ord, PartialOrd, Deserialize, Serialize)]
pub enum Role {
    // catch-all for whenever infallable conversion is needed
    #[default]
    Undefined,
    Manager,
    Owner,
    Editor,
    Reviewer,
    Reader,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Roles(pub(super) EnumSet<Role>);

mod impls;
