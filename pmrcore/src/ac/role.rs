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

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Roles(pub(super) EnumSet<Role>);

mod impls;
