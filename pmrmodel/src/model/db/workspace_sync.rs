use async_trait::async_trait;
use chrono::{LocalResult, TimeZone, Utc};
use pmrmodel_base::workspace_sync::{
    WorkspaceSyncRecord,
    WorkspaceSyncStatus,
};

use crate::backend::db::SqliteBackend;

#[async_trait]
pub trait WorkspaceSyncBackend {
    async fn begin_sync(&self, workspace_id: i64) -> anyhow::Result<i64>;
    async fn complete_sync(&self, id: i64, status: WorkspaceSyncStatus) -> anyhow::Result<bool>;
    async fn get_workspaces_sync_records(&self, workspace_id: i64) -> anyhow::Result<Vec<WorkspaceSyncRecord>>;
}

#[async_trait]
impl WorkspaceSyncBackend for SqliteBackend {
    async fn begin_sync(&self, workspace_id: i64) -> anyhow::Result<i64> {
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

    async fn complete_sync(&self, id: i64, status: WorkspaceSyncStatus) -> anyhow::Result<bool> {
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

    async fn get_workspaces_sync_records(&self, workspace_id: i64) -> anyhow::Result<Vec<WorkspaceSyncRecord>> {
        let recs = sqlx::query_as!(WorkspaceSyncRecord,
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
