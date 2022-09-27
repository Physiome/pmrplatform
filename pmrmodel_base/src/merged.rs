use serde::{Deserialize, Serialize};

use crate::git::{
    CommitInfo,
    PathObject,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspacePathInfo {
    pub workspace_id: i64,
    pub description: Option<String>,
    pub commit: CommitInfo,
    pub path: String,
    pub object: Option<PathObject>,
}
