use pmrmodel_base::workspace::WorkspaceRecord;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Deserialize, Serialize, PartialEq, Clone, derive_more::From
)]
pub struct JsonWorkspaceRecord {
    pub workspace: WorkspaceRecord,
    pub head_commit: Option<String>,
}
