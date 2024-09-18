use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
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

mod impls;
