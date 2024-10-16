use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    workspace::{
        WorkspaceAlias,
        traits::WorkspaceAliasBackend,
    },
};

use crate::{
    backend::db::SqliteBackend,
    chrono::Utc,
};

#[async_trait]
impl WorkspaceAliasBackend for SqliteBackend {

    async fn add_alias(
        &self,
        workspace_id: i64,
        alias: &str
    ) -> Result<i64, BackendError> {
        let ts = Utc::now().timestamp();
        let id = sqlx::query!(
            r#"
    INSERT INTO workspace_alias ( workspace_id, alias, created_ts )
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
    ) -> Result<Vec<WorkspaceAlias>, BackendError> {
        let recs = sqlx::query_as!(WorkspaceAlias,
            r#"
SELECT id, workspace_id, alias, created_ts
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
