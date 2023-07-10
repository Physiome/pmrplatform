use pmrmodel_base::workspace::WorkspaceSyncStatus;

use crate::model::db::workspace_sync::WorkspaceSyncBackend;

pub async fn fail_sync(
    backend: &impl WorkspaceSyncBackend,
    id: i64,
) -> Result<bool, sqlx::Error> {
    backend.complete_sync(id, WorkspaceSyncStatus::Error).await
}
