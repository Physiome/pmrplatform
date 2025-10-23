use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Citation {
    id: i64,
    identifier: String,
}

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct CitationLink {
    citation_id: i64,
    resource_path: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct CitationResourceSet {
    citation_id: i64,
    resource_paths: Vec<String>,
}

pub mod traits;
