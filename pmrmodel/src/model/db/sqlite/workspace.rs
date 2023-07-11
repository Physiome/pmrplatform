use async_trait::async_trait;
#[cfg(not(test))]
use chrono::Utc;
#[cfg(test)]
use crate::test::Utc;
use pmrmodel_base::workspace::{
    Workspace,
    Workspaces,
};

use crate::{
    backend::db::SqliteBackend,
    model::db::workspace::WorkspaceBackend,
};

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
INSERT INTO workspace (
    url,
    superceded_by_id,
    description,
    long_description,
    created_ts
)
VALUES ( ?1, ?2, ?3, ?4, ?5 )
            "#,
            url,
            None::<i64>,
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
    ) -> Result<Workspaces, sqlx::Error> {
        let recs = sqlx::query_as!(Workspace,
            r#"
SELECT
    id,
    url,
    superceded_by_id,
    description,
    long_description,
    created_ts
FROM
    workspace
ORDER BY
    id
            "#
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(recs.into())
    }

    async fn get_workspace_by_id(
        &self,
        id: i64,
    ) -> Result<Workspace, sqlx::Error> {
        // ignoring superceded_by_id for now?
        let rec = sqlx::query_as!(Workspace,
            r#"
SELECT
    id,
    url,
    superceded_by_id,
    description,
    long_description,
    created_ts
FROM
    workspace
WHERE
    id = ?1
            "#,
            id,
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(rec)
    }

    async fn list_workspace_by_url(
        &self,
        url: &str,
    ) -> Result<Workspaces, sqlx::Error> {
        let rec = sqlx::query_as!(Workspace,
            r#"
SELECT
    id,
    url,
    superceded_by_id,
    description,
    long_description,
    created_ts
FROM
    workspace
WHERE
    url = ?1
            "#,
            url,
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rec.into())
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrmodel_base::workspace::Workspace;
    use crate::backend::db::{
        Profile,
        SqliteBackend,
    };
    use crate::model::db::workspace::WorkspaceBackend;

    pub(crate) async fn make_example_workspace(
        backend: &dyn WorkspaceBackend,
    ) -> anyhow::Result<i64> {
        Ok(backend.add_workspace(
            "https://models.example.com".into(),
            "".into(),
            "".into(),
        ).await?)
    }

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Profile::Pmrapp)
            .await?;
        let id = make_example_workspace(&backend)
            .await?;
        let wb: &dyn WorkspaceBackend = &backend;
        let workspace = wb.get_workspace_by_id(id).await?;
        let answer = Workspace {
            id: 1,
            url: "https://models.example.com".into(),
            superceded_by_id: None,
            created_ts: 1234567890,
            description: Some("".into()),
            long_description: Some("".into()),
        };
        assert_eq!(workspace, answer);
        Ok(())
    }

    #[async_std::test]
    async fn test_list_by_url() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Profile::Pmrapp)
            .await?;
        // note this makes _two_ workspaces with the same url
        make_example_workspace(&backend).await?;
        make_example_workspace(&backend).await?;
        let wb: &dyn WorkspaceBackend = &backend;
        let workspaces = wb.list_workspace_by_url("https://models.example.com")
            .await?;
        assert_eq!(workspaces.len(), 2);
        Ok(())
    }

    #[async_std::test]
    async fn test_listing() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Profile::Pmrapp)
            .await?;
        let wb: &dyn WorkspaceBackend = &backend;
        make_example_workspace(wb).await?;
        make_example_workspace(wb).await?;
        make_example_workspace(wb).await?;
        assert_eq!(wb.list_workspaces().await?.len(), 3);

        Ok(())
    }

    #[async_std::test]
    async fn test_update() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Profile::Pmrapp)
            .await?;
        let id = make_example_workspace(&backend)
            .await?;
        let wb: &dyn WorkspaceBackend = &backend;
        assert!(wb.update_workspace(id, "title", "description").await?);

        let workspace = wb.get_workspace_by_id(id).await?;
        assert_eq!(workspace, Workspace {
            id: 1,
            url: "https://models.example.com".into(),
            superceded_by_id: None,
            created_ts: 1234567890,
            description: Some("title".into()),
            long_description: Some("description".into()),
        });
        Ok(())
    }

}
