use async_trait::async_trait;
use anyhow::bail;
use chrono::{LocalResult, TimeZone, Utc};

use enum_primitive::FromPrimitive;

use crate::backend::db::SqliteBackend;

#[async_trait]
pub trait WorkspaceSyncBackend {
    async fn begin_sync(&self, workspace_id: i64) -> anyhow::Result<i64>;
    async fn complete_sync(&self, id: i64, status: WorkspaceSyncStatus) -> anyhow::Result<bool>;
    async fn get_workspaces_sync_records(&self, workspace_id: i64) -> anyhow::Result<Vec<WorkspaceSyncRecord>>;
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum WorkspaceSyncStatus {
    Completed,
    Running,
    Error,
    Unknown = -1,
}
}

pub struct WorkspaceSyncRecord {
    pub id: i64,
    pub workspace_id: i64,
    pub start: i64,
    pub end: Option<i64>,
    pub status: i64,
}

impl std::fmt::Display for WorkspaceSyncRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} - {:?}",
            match Utc.timestamp_opt(self.start, 0) {
                LocalResult::Single(v) => v.to_rfc3339(),
                _ => "<invalid>".to_string(),
            },
            match self.end {
                Some(v) => match Utc.timestamp_opt(v, 0) {
                    LocalResult::Single(v) => v.to_rfc3339(),
                    _ => "<invalid>".to_string(),
                },
                None => "<nil>".to_string(),
            },
            WorkspaceSyncStatus::from_i64(self.status).unwrap_or(WorkspaceSyncStatus::Unknown),
        )
    }
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

pub async fn fail_sync(backend: &impl WorkspaceSyncBackend, id: i64, msg: String) -> anyhow::Result<()> {
    backend.complete_sync(id, WorkspaceSyncStatus::Error).await?;
    bail!(msg);
}
