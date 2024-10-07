use serde::{Deserialize, Serialize};
use super::user::User;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum Agent {
    Anonymous,
    User(User),
}

#[cfg(feature = "display")]
mod display;
mod impls;
