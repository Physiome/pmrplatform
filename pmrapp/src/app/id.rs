use serde::{Deserialize, Serialize};

/// Used to disambiguate a query for an alias or the real identifier
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Id {
    Aliased(String),
    Number(String),
}
