use serde::{Deserialize, Serialize};

/// The underlying core model for an alias.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Alias {
    pub kind: String,
    pub kind_id: i64,
    pub alias: String,
    pub created_ts: i64,
}

/// Newtype for `Vec<Alias>`
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Aliases(Vec<Alias>);

/// A collection of alias entries, identified by the kind label.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AliasEntries<T> {
    pub(crate) kind: String,
    pub(crate) entries: Vec<AliasEntry<T>>,
}

/// An alias entry for `T`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AliasEntry<T> {
    pub alias: String,
    pub entity: T,
}

/// The underlying core model that represents a request for an alias by
/// some user.  To be implemented.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AliasRequest {
    pub kind: String,
    pub kind_id: i64,
    pub alias: String,
    pub created_ts: i64,
    pub user_id: i64,
}

mod impls;
mod refs;
pub mod traits;

pub use refs::{
    AliasRef,
    AliasRefs,
};
