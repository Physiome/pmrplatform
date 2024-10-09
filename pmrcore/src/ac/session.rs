use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Session {
    pub token: SessionToken,
    pub user_id: i64,
    pub origin: String,
    pub created_ts: i64,
    pub last_active_ts: i64,
}

mod impls;
mod token;

pub use token::*;
