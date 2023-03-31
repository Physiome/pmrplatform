use async_trait::async_trait;
use chrono::Utc;
use pmrmodel_base::workspace::{
    WorkspaceRecord,
    WorkspaceRecords,
};
use std::io::Write;

use crate::backend::db::SqliteBackend;

#[async_trait]
pub trait WorkspaceBackend {
    async fn add_workspace(
        &self, url: &str, description: &str, long_description: &str
    ) -> anyhow::Result<i64>;
    async fn update_workspace(
        &self, id: i64, description: &str, long_description: &str
    ) -> anyhow::Result<bool>;
    async fn list_workspaces(&self) -> anyhow::Result<Vec<WorkspaceRecord>>;
    async fn get_workspace_by_id(&self, id: i64) -> anyhow::Result<WorkspaceRecord>;
    async fn get_workspace_by_url(&self, url: &str) -> anyhow::Result<WorkspaceRecord>;
}

pub fn stream_workspace_records_default(mut writer: impl Write, records: Vec<WorkspaceRecord>) -> std::result::Result<usize, std::io::Error> {
    let mut result: usize = 0;
    result += writer.write(b"id - url - description\n")?;
    for record in records {
        result += writer.write(format!("{}\n", record).as_bytes())?;
    }
    Ok(result)
}

pub fn stream_workspace_records_as_json(writer: impl Write, records: Vec<WorkspaceRecord>) -> Result<(), serde_json::Error> {
    serde_json::to_writer(writer, &WorkspaceRecords { workspaces: records })
}

#[async_trait]
impl WorkspaceBackend for SqliteBackend {
    async fn add_workspace(&self, url: &str, description: &str, long_description: &str) -> anyhow::Result<i64> {
        let ts = Utc::now().timestamp();

        let id = sqlx::query!(
            r#"
INSERT INTO workspace ( url, description, long_description, created )
VALUES ( ?1, ?2, ?3, ?4 )
            "#,
            url,
            description,
            long_description,
            ts,
        )
        .execute(&*self.pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    async fn update_workspace(&self, id: i64, description: &str, long_description: &str) -> anyhow::Result<bool> {
        let rows_affected = sqlx::query!(
            r#"
UPDATE workspace
SET description = ?1, long_description = ?2
WHERE id = ?3
            "#,
            description,
            long_description,
            id,
        )
        .execute(&*self.pool)
        .await?
        .rows_affected();

        Ok(rows_affected > 0)
    }

    async fn list_workspaces(&self) -> anyhow::Result<Vec<WorkspaceRecord>> {
        let recs = sqlx::query_as!(WorkspaceRecord,
            r#"
SELECT id, url, description
FROM workspace
ORDER BY id
            "#
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(recs)
    }

    async fn get_workspace_by_id(&self, id: i64) -> anyhow::Result<WorkspaceRecord> {
        // ignoring superceded_by_id for now?
        let rec = sqlx::query_as!(WorkspaceRecord,
            r#"
SELECT id, url, description
FROM workspace
WHERE id = ?1
            "#,
            id,
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(rec)
    }

    // XXX this assumes url is unique
    async fn get_workspace_by_url(&self, url: &str) -> anyhow::Result<WorkspaceRecord> {
        // ignoring superceded_by_id for now?
        let rec = sqlx::query_as!(WorkspaceRecord,
            r#"
SELECT id, url, description
FROM workspace
WHERE url = ?1
            "#,
            url,
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(rec)
    }
}
