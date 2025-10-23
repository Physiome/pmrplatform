use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Citation {
    pub id: i64,
    pub identifier: String,
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct CitationLink {
    pub citation_id: i64,
    pub resource_path: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CitationResourceSet {
    pub citation_id: i64,
    pub resource_paths: Vec<String>,
}

pub mod traits;
