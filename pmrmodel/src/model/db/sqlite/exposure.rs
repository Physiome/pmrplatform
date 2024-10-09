use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    exposure::{
        Exposure,
        Exposures,
        traits::ExposureBackend,
    },
};

use crate::{
    backend::db::SqliteBackend,
    chrono::Utc,
};

async fn insert_exposure_sqlite(
    sqlite: &SqliteBackend,
    description: Option<&str>,
    workspace_id: i64,
    workspace_tag_id: Option<i64>,
    commit_id: &str,
    default_file_id: Option<i64>,
) -> Result<i64, BackendError> {
    let created_ts = Utc::now().timestamp();
    let id = sqlx::query!(
        r#"
INSERT INTO exposure (
    description,
    workspace_id,
    workspace_tag_id,
    commit_id,
    created_ts,
    default_file_id
)
VALUES ( ?1, ?2, ?3, ?4, ?5, ?6 )
        "#,
        description,
        workspace_id,
        workspace_tag_id,
        commit_id,
        created_ts,
        default_file_id,
    )
    .execute(&*sqlite.pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

async fn get_exposure_by_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<Exposure, BackendError> {
    let rec = sqlx::query!(r#"
SELECT
    id,
    description,
    workspace_id,
    workspace_tag_id,
    commit_id,
    created_ts,
    default_file_id
FROM exposure
WHERE id = ?1
"#,
        id,
    )
    .map(|row| Exposure {
        id: row.id,
        description: row.description,
        workspace_id: row.workspace_id,
        workspace_tag_id: row.workspace_tag_id,
        commit_id: row.commit_id,
        created_ts: row.created_ts,
        default_file_id: row.default_file_id,
        // TODO map to files.
        files: None,
    })
    .fetch_one(&*sqlite.pool)
    .await?;
    Ok(rec)
}

async fn list_exposures_sqlite(
    sqlite: &SqliteBackend,
) -> Result<Exposures, BackendError> {
    let rec = sqlx::query!(r#"
SELECT
    id,
    description,
    workspace_id,
    workspace_tag_id,
    commit_id,
    created_ts,
    default_file_id
FROM exposure
"#,
    )
    .map(|row| Exposure {
        id: row.id,
        description: row.description,
        workspace_id: row.workspace_id,
        workspace_tag_id: row.workspace_tag_id,
        commit_id: row.commit_id,
        created_ts: row.created_ts,
        default_file_id: row.default_file_id,
        // won't have files.
        files: None,
    })
    .fetch_all(&*sqlite.pool)
    .await?;
    Ok(rec.into())
}

async fn list_exposures_for_workspace_sqlite(
    sqlite: &SqliteBackend,
    workspace_id: i64,
) -> Result<Exposures, BackendError> {
    let rec = sqlx::query!(r#"
SELECT
    id,
    description,
    workspace_id,
    workspace_tag_id,
    commit_id,
    created_ts,
    default_file_id
FROM exposure
WHERE workspace_id = ?1
"#,
        workspace_id,
    )
    .map(|row| Exposure {
        id: row.id,
        description: row.description,
        workspace_id: row.workspace_id,
        workspace_tag_id: row.workspace_tag_id,
        commit_id: row.commit_id,
        created_ts: row.created_ts,
        default_file_id: row.default_file_id,
        // won't have files.
        files: None,
    })
    .fetch_all(&*sqlite.pool)
    .await?;
    Ok(rec.into())
}

async fn set_default_file_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
    file_id: i64,
) -> Result<bool, BackendError> {
    let rows_affected = sqlx::query!(r#"
UPDATE exposure
SET default_file_id = ?2
WHERE id = ?1
    AND ?2 IN (
        SELECT id
        FROM exposure_file
        WHERE exposure_id = ?1
    )
"#,
        id,
        file_id,
    )
    .execute(&*sqlite.pool)
    .await?
    .rows_affected();
    Ok(rows_affected > 0)
}

#[async_trait]
impl ExposureBackend for SqliteBackend {
    async fn insert(
        &self,
        description: Option<&str>,
        workspace_id: i64,
        workspace_tag_id: Option<i64>,
        commit_id: &str,
        default_file_id: Option<i64>,
    ) -> Result<i64, BackendError>{
        insert_exposure_sqlite(
            &self,
            description,
            workspace_id,
            workspace_tag_id,
            commit_id,
            default_file_id,
        ).await
    }

    async fn list(
        &self,
    ) -> Result<Exposures, BackendError> {
        list_exposures_sqlite(
            &self,
        ).await
    }

    async fn list_for_workspace(
        &self,
        workspace_id: i64,
    ) -> Result<Exposures, BackendError> {
        list_exposures_for_workspace_sqlite(
            &self,
            workspace_id,
        ).await
    }

    async fn get_id(
        &self,
        id: i64,
    ) -> Result<Exposure, BackendError> {
        get_exposure_by_id_sqlite(
            &self,
            id,
        ).await
    }

    async fn set_default_file(
        &self,
        id: i64,
        file_id: i64,
    ) -> Result<bool, BackendError> {
        set_default_file_sqlite(
            &self,
            id,
            file_id,
        ).await
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrcore::{
        exposure::{
            Exposure,
            traits::{
                Exposure as _,
                ExposureBackend,
            },
        },
        platform::MCPlatform,
        workspace::traits::Workspace,
    };
    use crate::backend::db::{
        MigrationProfile,
        SqliteBackend,
    };
    use crate::model::db::sqlite::workspace::testing::make_example_workspace;

    pub(crate) async fn make_example_exposure(
        backend: &dyn ExposureBackend,
        workspace_id: i64,
    ) -> anyhow::Result<i64> {
        let description = format!("Exposure for Workspace {workspace_id}");
        Ok(backend.insert(
            Some(&description),
            workspace_id,
            None,
            "abcdef".into(),
            None,
        ).await?)
    }

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrapp)
            .await?;

        let workspace_id = make_example_workspace(&backend).await?;
        let id = make_example_exposure(&backend, workspace_id).await?;
        let exposure = ExposureBackend::get_id(
            &backend, id
        ).await?;
        assert_eq!(exposure, Exposure {
            id: 1,
            description: Some("Exposure for Workspace 1".into()),
            workspace_id: 1,
            workspace_tag_id: None,
            commit_id: "abcdef".into(),
            created_ts: 1234567890,
            default_file_id: None,
            files: None,
            // files: Some([].to_vec().into()),
        });

        let workspace = MCPlatform::get_workspace(&backend, id).await?;
        assert_eq!(workspace.id(), workspace_id);
        let exposures = workspace.exposures().await?;
        assert_eq!(exposures.len(), 1);
        assert_eq!(exposures[0].id(), exposure.id);
        Ok(())
    }

    #[async_std::test]
    async fn test_get_exposure_workspace() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrapp)
            .await?;

        let w1 = make_example_workspace(&backend).await?;
        let w2 = make_example_workspace(&backend).await?;
        make_example_exposure(&backend, w1).await?;
        make_example_exposure(&backend, w1).await?;
        make_example_exposure(&backend, w2).await?;
        make_example_exposure(&backend, w2).await?;
        make_example_exposure(&backend, w2).await?;
        make_example_exposure(&backend, w2).await?;

        let eb: &dyn ExposureBackend = &backend;
        assert_eq!(2, eb.list_for_workspace(w1).await?.len());
        assert_eq!(4, eb.list_for_workspace(w2).await?.len());

        Ok(())
    }

}
