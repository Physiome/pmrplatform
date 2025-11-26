use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};
use crate::exposure::Exposures;

#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Workspace {
    pub id: i64,
    pub url: String,
    pub superceded_by_id: Option<i64>,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub created_ts: i64,

    pub exposures: Option<Exposures>,
}

#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Workspaces(Vec<Workspace>);

#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
#[derive(Debug, PartialEq, FromPrimitive)]
#[repr(i64)]
pub enum WorkspaceSyncStatus {
    Completed,
    Running,
    Error,
    #[num_enum(default)]
    Unknown = -1,
}

#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct WorkspaceSync {
    pub id: i64,
    pub workspace_id: i64,
    pub start: i64,
    pub end: Option<i64>,
    pub status: i64,
}

#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct WorkspaceTag {
    pub id: i64,
    pub workspace_id: i64,
    pub name: String,
    pub commit_id: String,
}

#[cfg(feature = "display")]
mod display;
mod impls;
pub mod traits;
mod refs;

pub use refs::{
    WorkspaceRef,
    WorkspaceRefs,
};
