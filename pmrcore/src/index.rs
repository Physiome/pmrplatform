use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

/// The underlying raw entity for the kind of the index
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
pub struct IndexTerms {
    pub kind: IdxKind,
    pub terms: Vec<String>,
}

/// A listing of resources for a particular term under a particular index identified by [`IdxKind`].
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
pub struct IndexResourceSet {
    pub kind: IdxKind,
    pub term: String,
    // TODO perhaps resource_path may be enclosed in an option to denote the term is unknown
    // TODO need to have an API that turn the resource_path into a fully form record type that
    // will provide the actual alias associated with any given path.
    pub resource_paths: Vec<String>,
}

/// A listing of resources for a particular term under a particular index identified by [`IdxKind`],
/// with the details of the kind and terms provided for the resource
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
pub struct IndexResourceDetailedSet {
    pub kind: IdxKind,
    pub term: String,
    // TODO perhaps resource_path may be enclosed in an option to denote the term is unknown
    // TODO need to have an API that turn the resource_path into a fully form record type that
    // will provide the actual alias associated with any given path.
    pub resource_paths: Vec<ResourceKindedTerms>,
}

/// A listing of resources for a particular term under a particular index identified by [`IdxKind`].
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
pub struct ResourceKindedTerms {
    pub resource_path: String,
    pub data: BTreeMap<String, Vec<String>>,
}

pub mod traits;
