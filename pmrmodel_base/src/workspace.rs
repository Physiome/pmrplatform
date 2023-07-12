use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Workspace {
    pub id: i64,
    pub url: String,
    pub superceded_by_id: Option<i64>,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub created_ts: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Workspaces(Vec<Workspace>);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct WorkspaceAlias {
    pub id: i64,
    pub workspace_id: i64,
    pub alias: String,
    pub created_ts: i64,
}

#[derive(Debug, PartialEq, FromPrimitive)]
#[repr(i64)]
pub enum WorkspaceSyncStatus {
    Completed,
    Running,
    Error,
    #[num_enum(default)]
    Unknown = -1,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct WorkspaceSync {
    pub id: i64,
    pub workspace_id: i64,
    pub start: i64,
    pub end: Option<i64>,
    pub status: i64,
}

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
