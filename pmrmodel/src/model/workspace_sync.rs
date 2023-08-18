use pmrcore::{
    error::BackendError,
    workspace::{
        WorkspaceSyncStatus,
        traits::WorkspaceSyncBackend,
    },
};

pub async fn fail_sync(
    backend: &dyn WorkspaceSyncBackend,
    id: i64,
) -> Result<bool, BackendError> {
    backend.complete_sync(id, WorkspaceSyncStatus::Error).await
}
