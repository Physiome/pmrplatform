use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};
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

// TODO Rather than being reconstructed from the normalized form, there should be a denormalized form
// of the following structure in the database somewhere.  I would assume the data be serialized and
// stored as a text/blob.
//
// Mainly, this would significantly speed up the lookup of data associated with resource_path, but
// also provide injection of data associated with the parent but normally not associated with searches.
// Though ideally the denormalized form should record the updated timestamp (in at least microsecond
// precision) and also the keys that are actually from the object.
//
// Naturally, this will help eliminate duplicate data that's stored as part of the parent (e.g. the
// workflow state/timestamp) that's common to all children items, so that during resource_path lookup
// those data will be provided for the children.
//
// Updating the index should now include a last-updated field, but this cache (in)validation strategy
// will need to be re-evaluated at time of implementation.
//
// A consideration will need to be made on what to do with text handling, as those are bulky fields
// so an additional lookup may still be required for that.
/// A listing of resources for a particular term under a particular index identified by [`IdxKind`].
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
pub struct ResourceKindedTerms {
    pub resource_path: String,
    pub data: BTreeMap<String, Vec<String>>,
}

/// A brief about the resource.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
pub struct ResourceBrief {
    pub resource_path: String,
    pub title: Option<String>,
    pub brief: Option<String>,
}

/// Used for search/query.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
pub struct Filter {
    pub kind: String,
    pub term: String,
}

/// Used for search/query.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
pub struct Query {
    pub query: Option<String>,
    pub filters: Vec<Filter>,
}

/// A memory cache of `ResourceKindedTerms` stored within some `IndexBackend` that has been retrieved.
#[derive(Clone, Debug, Default)]
pub struct ResourceKindedTermsCache<B>
where
    B: ?Sized,
{
    backend: Arc<B>,
    heap: Arc<RwLock<BTreeMap<String, BTreeMap<String, Vec<String>>>>>,
}

/// A generic implementation to enable `IndexCoreCache` for some `IndexCoreBackend`.
///
/// Typically some database backend that implements `IndexCoreBackend` may also implement `IndexCoreCache`.
/// The generic `IndexBackend` implementation is only implemented for those that implement `IndexCoreBackend`,
/// with the caching unused.  This struct wraps some `IndexBackend` that also implements `IndexCoreCache` so
/// that the implemented caching methodology for the backend (usually on disk) may be used and provided by this
/// generic implementation.
#[derive(Clone, Debug, Default)]
pub struct IndexBackendCache<B>
where
    B: ?Sized,
{
    backend: Arc<B>,
}

mod impls;
pub mod traits;
