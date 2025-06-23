use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Alias {
    pub kind: String,
    pub kind_id: i64,
    pub alias: String,
    pub created_ts: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AliasRequest {
    pub kind: String,
    pub kind_id: i64,
    pub alias: String,
    pub created_ts: i64,
    pub user_id: i64,
}

pub mod traits;
