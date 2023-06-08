use anyhow::bail;
use pmrmodel_base::workspace_sync::WorkspaceSyncStatus;

use crate::model::db::workspace_sync::WorkspaceSyncBackend;

pub async fn fail_sync(backend: &impl WorkspaceSyncBackend, id: i64, msg: String) -> anyhow::Result<()> {
    backend.complete_sync(id, WorkspaceSyncStatus::Error).await?;
    bail!(msg);
}
