use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Citation {
    pub id: i64,
    pub identifier: String,
}

pub mod traits;
