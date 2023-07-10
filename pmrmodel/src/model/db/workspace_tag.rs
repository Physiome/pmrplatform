use async_trait::async_trait;
use pmrmodel_base::workspace::WorkspaceTag;

use crate::backend::db::SqliteBackend;

#[async_trait]
pub trait WorkspaceTagBackend {
    async fn index_workspace_tag(
        &self,
        workspace_id: i64,
        name: &str,
        commit_id: &str,
    ) -> Result<i64, sqlx::Error>;
    async fn get_workspace_tags(
        &self,
        workspace_id: i64,
    ) -> Result<Vec<WorkspaceTag>, sqlx::Error>;
}

#[async_trait]
impl WorkspaceTagBackend for SqliteBackend {

    async fn index_workspace_tag(
        &self,
        workspace_id: i64,
        name: &str,
        commit_id: &str,
    ) -> Result<i64, sqlx::Error> {
        let id = sqlx::query!(
            r#"
    INSERT INTO workspace_tag ( workspace_id, name, commit_id )
    VALUES ( ?1, ?2, ?3 )
    ON CONFLICT (workspace_id, name, commit_id) DO NOTHING
            "#,
            workspace_id,
            name,
            commit_id,
        )
        .execute(&*self.pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }
    // TODO create test so that the unique indexes are done correctly

    async fn get_workspace_tags(
        &self,
        workspace_id: i64,
    ) -> Result<Vec<WorkspaceTag>, sqlx::Error> {
        let recs = sqlx::query_as!(WorkspaceTag,
            r#"
    SELECT id, workspace_id, name, commit_id
    FROM workspace_tag
    WHERE workspace_id = ?1
            "#,
            workspace_id,
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(recs)
    }

}
