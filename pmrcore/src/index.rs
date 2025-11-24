use serde::{Deserialize, Serialize};

/// The underlying raw entity for the kind of the index
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IdxKind {
    pub id: i64,
    pub description: String,
}

/// The underlying raw entity for an index entry
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IdxEntry {
    pub id: i64,
    pub idx_kind_id: i64,
    pub term: String,
}

/// The underlying raw link to a resource for a term from an index entry
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IdxEntryLink {
    pub idx_entry_id: i64,
    pub resource_path: String,
}

/// A listing of terms for a particular index identified by [`IdxKind`].
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IndexTerms {
    pub kind: IdxKind,
    pub terms: Vec<String>,
}

/// A listing of resources for a particular term under a particular index identified by [`IdxKind`].
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct IndexResourceSet {
    pub kind: IdxKind,
    pub term: String,
    pub resource_paths: Vec<String>,
}

pub mod traits;
