use async_trait::async_trait;
#[cfg(not(test))]
use chrono::Utc;
#[cfg(test)]
use crate::test::Utc;
use pmrmodel_base::exposure::{
    Exposure,
    Exposures,
    // ExposureFile,
    // ExposureFiles,
    // ExposureFileView,
    // ExposureFileViews,
};

use crate::backend::db::SqliteBackend;

async fn add_exposure_sqlite(
    sqlite: &SqliteBackend,
    workspace_id: i64,
    workspace_tag_id: Option<i64>,
    commit_id: String,
    root_exposure_file_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
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
) -> Result<Exposure, sqlx::Error> {
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
) -> Result<Exposures, sqlx::Error> {
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
pub trait ExposureBackend {
    async fn add_exposure(
        &self,
        workspace_id: i64,
        workspace_tag_id: Option<i64>,
        commit_id: String,
        root_exposure_file_id: Option<i64>,
    ) -> Result<i64, sqlx::Error>;
    async fn list_exposures_for_workspace(
        &self,
        workspace_id: i64,
    ) -> Result<Exposures, sqlx::Error>;
    async fn get_exposure_by_id(
        &self,
        id: i64,
    ) -> Result<Exposure, sqlx::Error>;
}

#[async_trait]
impl ExposureBackend for SqliteBackend {
    async fn add_exposure(
        &self,
        workspace_id: i64,
        workspace_tag_id: Option<i64>,
        commit_id: String,
        root_exposure_file_id: Option<i64>,
    ) -> Result<i64, sqlx::Error>{
        add_exposure_sqlite(
            &self,
            workspace_id,
            workspace_tag_id,
            commit_id,
            root_exposure_file_id,
        ).await
    }

    async fn list_exposures_for_workspace(
        &self,
        workspace_id: i64,
    ) -> Result<Exposures, sqlx::Error> {
        list_exposures_for_workspace_sqlite(
            &self,
            workspace_id,
        ).await
    }

    async fn get_exposure_by_id(
        &self,
        id: i64,
    ) -> Result<Exposure, sqlx::Error> {
        get_exposure_by_id_sqlite(
            &self,
            id,
        ).await
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrmodel_base::exposure::Exposure;
    use crate::backend::db::{
        Profile,
        SqliteBackend,
    };
    use crate::model::db::exposure::ExposureBackend;
    use crate::model::db::workspace::testing::make_example_workspace;

    pub(crate) async fn make_example_exposure(
        backend: &dyn ExposureBackend,
        workspace_id: i64,
    ) -> anyhow::Result<i64> {
        Ok(backend.add_exposure(
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
        let exposure = ExposureBackend::get_exposure_by_id(
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

}
