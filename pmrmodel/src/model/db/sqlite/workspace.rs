use async_trait::async_trait;
#[cfg(not(test))]
use chrono::Utc;
#[cfg(test)]
use crate::test::Utc;
use pmrmodel_base::{
    error::BackendError,
    workspace::{
        Workspace,
        Workspaces,
        traits::WorkspaceBackend,
    }
};

use crate::{
    backend::db::SqliteBackend,
};


async fn add_workspace_sqlite(
    backend: &SqliteBackend,
    url: &str,
    description: &str,
    long_description: &str,
) -> Result<i64, BackendError> {
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
    .execute(&*backend.pool)
    .await?
    .last_insert_rowid();

    Ok(id)
}

async fn update_workspace_sqlite(
    backend: &SqliteBackend,
    id: i64,
    description: &str,
    long_description: &str,
) -> Result<bool, BackendError> {
    let rows_affected = sqlx::query!(r#"
UPDATE
    workspace
SET
    description = ?1,
    long_description = ?2
WHERE
    id = ?3"#,
        description,
        long_description,
        id,
    )
    .execute(&*backend.pool)
    .await?
    .rows_affected();
    Ok(rows_affected > 0)
}

async fn list_workspaces_sqlite(
    backend: &SqliteBackend,
) -> Result<Workspaces, BackendError> {
    let recs = sqlx::query!(r#"
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
    .map(|row| Workspace {
        id: row.id,
        url: row.url,
        superceded_by_id: row.superceded_by_id,
        description: row.description,
        long_description: row.long_description,
        created_ts: row.created_ts,
        exposures: None,
    })
    .fetch_all(&*backend.pool)
    .await?;
    Ok(recs.into())
}

async fn get_workspace_by_id_sqlite(
    backend: &SqliteBackend,
    id: i64,
) -> Result<Workspace, BackendError> {
    // ignoring superceded_by_id for now?
    let rec = sqlx::query!(r#"
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
    .map(|row| Workspace {
        id: row.id,
        url: row.url,
        superceded_by_id: row.superceded_by_id,
        description: row.description,
        long_description: row.long_description,
        created_ts: row.created_ts,
        exposures: None,
    })
    .fetch_one(&*backend.pool)
    .await?;
    Ok(rec)
}

async fn list_workspaces_by_url_sqlite(
    backend: &SqliteBackend,
    url: &str,
) -> Result<Workspaces, BackendError> {
    let recs = sqlx::query!(r#"
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
    .map(|row| Workspace {
        id: row.id,
        url: row.url,
        superceded_by_id: row.superceded_by_id,
        description: row.description,
        long_description: row.long_description,
        created_ts: row.created_ts,
        exposures: None,
    })
    .fetch_all(&*backend.pool)
    .await?;
    Ok(recs.into())
}

#[async_trait]
impl WorkspaceBackend for SqliteBackend {
    async fn add_workspace(
        &self,
        url: &str,
        description: &str,
        long_description: &str,
    ) -> Result<i64, BackendError> {
        add_workspace_sqlite(
            &self,
            url,
            description,
            long_description,
        ).await
    }

    async fn update_workspace(
        &self,
        id: i64,
        description: &str,
        long_description: &str,
    ) -> Result<bool, BackendError> {
        update_workspace_sqlite(
            &self,
            id,
            description,
            long_description,
        ).await
    }

    async fn list_workspaces(
        &self,
    ) -> Result<Workspaces, BackendError> {
        list_workspaces_sqlite(&self).await
    }

    async fn get_workspace_by_id(
        &self,
        id: i64,
    ) -> Result<Workspace, BackendError> {
        get_workspace_by_id_sqlite(&self, id).await
    }

    async fn list_workspace_by_url(
        &self,
        url: &str,
    ) -> Result<Workspaces, BackendError> {
        list_workspaces_by_url_sqlite(&self, url).await
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrmodel_base::workspace::{
        Workspace,
        traits::WorkspaceBackend,
    };
    use crate::backend::db::{
        Profile,
        SqliteBackend,
    };

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
            exposures: None,
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
            exposures: None,
        });
        Ok(())
    }

}
