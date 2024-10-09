use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    workspace::{
        WorkspaceSync,
        WorkspaceSyncStatus,
        traits::WorkspaceSyncBackend,
    },
};

use crate::{
    backend::db::SqliteBackend,
    chrono::Utc,
};

#[async_trait]
impl WorkspaceSyncBackend for SqliteBackend {
    async fn begin_sync(
        &self,
        workspace_id: i64,
    ) -> Result<i64, BackendError> {
        let ts = Utc::now().timestamp();

        let id = sqlx::query!(
            r#"
    INSERT INTO workspace_sync ( workspace_id, start, status )
    VALUES ( ?1, ?2, ?3 )
            "#,
            workspace_id,
            ts,
            WorkspaceSyncStatus::Running as i32,
        )
        .execute(&*self.pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    async fn complete_sync(
        &self,
        id: i64,
        status: WorkspaceSyncStatus,
    ) -> Result<bool, BackendError> {
        let ts = Utc::now().timestamp();
        let status_ = status as i32;

        let rows_affected = sqlx::query!(
            r#"
    UPDATE workspace_sync
    SET end = ?1, status = ?2
    WHERE id = ?3
            "#,
            ts,
            status_,
            id,
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    async fn get_workspaces_sync_records(
        &self,
        workspace_id: i64,
    ) -> Result<Vec<WorkspaceSync>, BackendError> {
        let recs = sqlx::query_as!(WorkspaceSync,
            r#"
SELECT id, workspace_id, start, end, status
FROM workspace_sync
WHERE workspace_id = ?1
            "#,
            workspace_id,
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(recs)
    }
}
