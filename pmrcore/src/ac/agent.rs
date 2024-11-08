use serde::{Deserialize, Serialize};
use super::user::User;

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum Agent {
    #[default]
    Anonymous,
    User(User),
}

#[cfg(feature = "display")]
mod display;
mod impls;
