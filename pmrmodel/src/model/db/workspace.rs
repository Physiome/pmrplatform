use async_trait::async_trait;
use chrono::Utc;
use pmrmodel_base::workspace::WorkspaceRecord;

use crate::backend::db::SqliteBackend;

#[async_trait]
pub trait WorkspaceBackend {
    async fn add_workspace(
        &self,
        url: &str,
        description: &str,
        long_description: &str,
    ) -> Result<i64, sqlx::Error>;
    async fn update_workspace(
        &self,
        id: i64,
        description: &str,
        long_description: &str,
    ) -> Result<bool, sqlx::Error>;
    async fn list_workspaces(
        &self,
    ) -> Result<Vec<WorkspaceRecord>, sqlx::Error>;
    async fn get_workspace_by_id(
        &self,
        id: i64,
    ) -> Result<WorkspaceRecord, sqlx::Error>;
    async fn get_workspace_by_url(
        &self,
        url: &str,
    ) -> Result<WorkspaceRecord, sqlx::Error>;
}

#[async_trait]
impl WorkspaceBackend for SqliteBackend {
    async fn add_workspace(
        &self,
        url: &str,
        description: &str,
        long_description: &str,
    ) -> Result<i64, sqlx::Error> {
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

    async fn update_workspace(
        &self,
        id: i64,
        description: &str,
        long_description: &str,
    ) -> Result<bool, sqlx::Error> {
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

    async fn list_workspaces(
        &self,
    ) -> Result<Vec<WorkspaceRecord>, sqlx::Error> {
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

    async fn get_workspace_by_id(
        &self,
        id: i64,
    ) -> Result<WorkspaceRecord, sqlx::Error> {
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
    async fn get_workspace_by_url(
        &self,
        url: &str,
    ) -> Result<WorkspaceRecord, sqlx::Error> {
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
