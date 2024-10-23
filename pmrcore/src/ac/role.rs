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

pub struct Roles(pub(super) EnumSet<Role>);

mod impls;
