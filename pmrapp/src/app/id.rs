use serde::{Deserialize, Serialize};

/// Used to disambiguate a query for an alias or the real identifier
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Id {
    Aliased(String),
    Number(String),
}
