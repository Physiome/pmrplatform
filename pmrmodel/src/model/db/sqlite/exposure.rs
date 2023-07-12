use async_trait::async_trait;
#[cfg(not(test))]
use chrono::Utc;
#[cfg(test)]
use crate::test::Utc;
use pmrmodel_base::{
    error::BackendError,
    exposure::{
        Exposure,
        Exposures,
        // ExposureFile,
        // ExposureFiles,
        // ExposureFileView,
        // ExposureFileViews,
        traits::ExposureBackend,
    },
};

use crate::{
    backend::db::SqliteBackend,
};

async fn insert_exposure_sqlite(
    sqlite: &SqliteBackend,
    workspace_id: i64,
    workspace_tag_id: Option<i64>,
    commit_id: String,
    root_exposure_file_id: Option<i64>,
) -> Result<i64, BackendError> {
    let created_ts = Utc::now().timestamp();
    let id = sqlx::query!(
        r#"
INSERT INTO exposure (
    workspace_id,
    workspace_tag_id,
    commit_id,
    created_ts,
    root_exposure_file_id
)
VALUES ( ?1, ?2, ?3, ?4, ?5 )
        "#,
        workspace_id,
        workspace_tag_id,
        commit_id,
        created_ts,
        root_exposure_file_id,
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
    workspace_id,
    workspace_tag_id,
    commit_id,
    created_ts,
    root_exposure_file_id
FROM exposure
WHERE id = ?1
"#,
        id,
    )
    .map(|row| Exposure {
        id: row.id,
        workspace_id: row.workspace_id,
        workspace_tag_id: row.workspace_tag_id,
        commit_id: row.commit_id,
        created_ts: row.created_ts,
        root_exposure_file_id: row.root_exposure_file_id,
        // TODO map to files.
        files: None,
    })
    .fetch_one(&*sqlite.pool)
    .await?;
    Ok(rec)
}

async fn list_exposures_for_workspace_sqlite(
    sqlite: &SqliteBackend,
    workspace_id: i64,
) -> Result<Exposures, BackendError> {
    let rec = sqlx::query!(r#"
SELECT
    id,
    workspace_id,
    workspace_tag_id,
    commit_id,
    created_ts,
    root_exposure_file_id
FROM exposure
WHERE workspace_id = ?1
"#,
        workspace_id,
    )
    .map(|row| Exposure {
        id: row.id,
        workspace_id: row.workspace_id,
        workspace_tag_id: row.workspace_tag_id,
        commit_id: row.commit_id,
        created_ts: row.created_ts,
        root_exposure_file_id: row.root_exposure_file_id,
        // won't have files.
        files: None,
    })
    .fetch_all(&*sqlite.pool)
    .await?;
    Ok(rec.into())
}

#[async_trait]
impl ExposureBackend for SqliteBackend {
    async fn insert(
        &self,
        workspace_id: i64,
        workspace_tag_id: Option<i64>,
        commit_id: String,
        root_exposure_file_id: Option<i64>,
    ) -> Result<i64, BackendError>{
        insert_exposure_sqlite(
            &self,
            workspace_id,
            workspace_tag_id,
            commit_id,
            root_exposure_file_id,
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
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrmodel_base::exposure::Exposure;
    use pmrmodel_base::exposure::traits::ExposureBackend;
    use crate::backend::db::{
        Profile,
        SqliteBackend,
    };
    use crate::model::db::sqlite::workspace::testing::make_example_workspace;

    pub(crate) async fn make_example_exposure(
        backend: &dyn ExposureBackend,
        workspace_id: i64,
    ) -> anyhow::Result<i64> {
        Ok(backend.insert(
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
            .run_migration_profile(Profile::Pmrapp)
            .await?;

        let workspace_id = make_example_workspace(&backend).await?;
        let id = make_example_exposure(&backend, workspace_id).await?;
        let exposure = ExposureBackend::get_id(
            &backend, id
        ).await?;
        assert_eq!(exposure, Exposure {
            id: 1,
            workspace_id: 1,
            workspace_tag_id: None,
            commit_id: "abcdef".into(),
            created_ts: 1234567890,
            root_exposure_file_id: None,
            files: None,
            // files: Some([].to_vec().into()),
        });
        Ok(())
    }

    #[async_std::test]
    async fn test_get_exposure_workspace() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Profile::Pmrapp)
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
