use serde::{Deserialize, Serialize};
use super::user::User;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Agent {
    Anonymous,
    User(User),
}

mod impls;
