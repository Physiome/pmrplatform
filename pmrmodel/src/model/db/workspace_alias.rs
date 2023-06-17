use async_trait::async_trait;
use chrono::Utc;
use std::fmt;

use crate::backend::db::SqliteBackend;

#[async_trait]
pub trait WorkspaceAliasBackend {
    async fn add_alias(
        &self,
        workspace_id: i64,
        alias: &str,
    ) -> Result<i64, sqlx::Error>;
    async fn get_aliases(
        &self,
        workspace_id: i64,
    ) -> Result<Vec<WorkspaceAliasRecord>, sqlx::Error>;
}

pub struct WorkspaceAliasRecord {
    pub id: i64,
    pub workspace_id: i64,
    pub alias: String,
    pub created: i64,
}

impl std::fmt::Display for WorkspaceAliasRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {}",
            &self.workspace_id,
            &self.alias,
        )
    }
}

#[async_trait]
impl WorkspaceAliasBackend for SqliteBackend {

    async fn add_alias(
        &self,
        workspace_id: i64,
        alias: &str
    ) -> Result<i64, sqlx::Error> {
        let ts = Utc::now().timestamp();
        let id = sqlx::query!(
            r#"
    INSERT INTO workspace_alias ( workspace_id, alias, created )
    VALUES ( ?1, ?2, ?3 )
    ON CONFLICT (workspace_id, alias) DO NOTHING
            "#,
            workspace_id,
            alias,
            ts,
        )
        .execute(&*self.pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    async fn get_aliases(
        &self,
        workspace_id: i64,
    ) -> Result<Vec<WorkspaceAliasRecord>, sqlx::Error> {
        let recs = sqlx::query_as!(WorkspaceAliasRecord,
            r#"
    SELECT id, workspace_id, alias, created
    FROM workspace_alias
    WHERE workspace_id = ?1
            "#,
            workspace_id,
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(recs)
    }

}
