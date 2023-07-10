use pmrmodel_base::workspace::Workspace;
use serde::{Deserialize, Serialize};

// TODO unify this with main base model.
#[derive(
    Debug, Deserialize, Serialize, PartialEq, Clone, derive_more::From
)]
pub struct JsonWorkspaceRecord {
    pub workspace: Workspace,
    pub head_commit: Option<String>,
}
