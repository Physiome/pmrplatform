use async_trait::async_trait;
use pmrmodel_base::{
    error::BackendError,
    exposure::{
        ExposureFile,
        ExposureFiles,
        traits::ExposureFileBackend,
    },
};

use crate::{
    backend::db::SqliteBackend,
};

async fn insert_exposure_file_sqlite(
    sqlite: &SqliteBackend,
    exposure_id: i64,
    workspace_file_path: &str,
    default_view_id: Option<i64>,
) -> Result<i64, BackendError> {
    let id = sqlx::query!(
        r#"
INSERT INTO exposure_file (
    exposure_id,
    workspace_file_path,
    default_view_id
)
VALUES ( ?1, ?2, ?3 )
        "#,
        exposure_id,
        workspace_file_path,
        default_view_id,
    )
    .execute(&*sqlite.pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

async fn get_exposure_file_by_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<ExposureFile, BackendError> {
    let rec = sqlx::query!(r#"
SELECT
    id,
    exposure_id,
    workspace_file_path,
    default_view_id
FROM exposure_file
WHERE id = ?1
"#,
        id,
    )
    .map(|row| ExposureFile {
        id: row.id,
        exposure_id: row.exposure_id,
        workspace_file_path: row.workspace_file_path,
        default_view_id: row.default_view_id,
        views: None,
    })
    .fetch_one(&*sqlite.pool)
    .await?;
    Ok(rec)
}

async fn list_exposure_files_for_exposure_sqlite(
    sqlite: &SqliteBackend,
    workspace_id: i64,
) -> Result<ExposureFiles, BackendError> {
    let rec = sqlx::query!(r#"
SELECT
    id,
    exposure_id,
    workspace_file_path,
    default_view_id
FROM exposure_file
WHERE exposure_id = ?1
"#,
        workspace_id,
    )
    .map(|row| ExposureFile {
        id: row.id,
        exposure_id: row.exposure_id,
        workspace_file_path: row.workspace_file_path,
        default_view_id: row.default_view_id,
        views: None,
    })
    .fetch_all(&*sqlite.pool)
    .await?;
    Ok(rec.into())
}

async fn set_default_view_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
    view_id: i64,
) -> Result<bool, BackendError> {
    let rows_affected = sqlx::query!(r#"
UPDATE exposure_file
SET default_view_id = ?2
WHERE id = ?1
    AND ?2 IN (
        SELECT id
        FROM exposure_file_view
        WHERE exposure_file_id = ?1
    )
"#,
        id,
        view_id,
    )
    .execute(&*sqlite.pool)
    .await?
    .rows_affected();
    Ok(rows_affected > 0)
}

#[async_trait]
impl ExposureFileBackend for SqliteBackend {
    async fn insert(
        &self,
        exposure_id: i64,
        workspace_file_path: &str,
        default_view_id: Option<i64>,
    ) -> Result<i64, BackendError>{
        insert_exposure_file_sqlite(
            &self,
            exposure_id,
            workspace_file_path,
            default_view_id,
        ).await
    }

    async fn list_for_exposure(
        &self,
        exposure_id: i64,
    ) -> Result<ExposureFiles, BackendError> {
        list_exposure_files_for_exposure_sqlite(
            &self,
            exposure_id,
        ).await
    }

    async fn get_id(
        &self,
        id: i64,
    ) -> Result<ExposureFile, BackendError> {
        get_exposure_file_by_id_sqlite(
            &self,
            id,
        ).await
    }

    async fn set_default_view(
        &self,
        id: i64,
        view_id: i64
    ) -> Result<bool, BackendError> {
        set_default_view_sqlite(
            &self,
            id,
            view_id,
        ).await
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrmodel_base::{
        exposure::{
            ExposureFile,
            traits::Backend,
            traits::Exposure as _,
            traits::ExposureBackend,
            traits::ExposureFile as _,
            traits::ExposureFileBackend,
        },
    };
    use crate::backend::db::{
        Profile,
        SqliteBackend,
    };
    use crate::model::db::sqlite::workspace::testing::make_example_workspace;
    use crate::model::db::sqlite::exposure::testing::make_example_exposure;

    pub(crate) async fn make_example_exposure_file(
        backend: &dyn ExposureFileBackend,
        exposure_id: i64,
        workspace_file_path: &str,
    ) -> anyhow::Result<i64> {
        Ok(backend.insert(
            exposure_id,
            workspace_file_path,
            None,
        ).await?)
    }

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Profile::Pmrapp)
            .await?;

        let exposure_id = make_example_exposure(
            &backend,
            make_example_workspace(&backend).await?,
        ).await?;
        let id = make_example_exposure_file(
            &backend, exposure_id, "README.md",
        ).await?;
        let ef_backend: &dyn ExposureFileBackend = &backend;
        let exposure_file = ef_backend.get_id(id).await?;
        assert_eq!(exposure_file, ExposureFile {
            id: 1,
            exposure_id: 1,
            workspace_file_path: "README.md".into(),
            default_view_id: None,
            views: None,
        });
        Ok(())
    }

    #[async_std::test]
    async fn test_using_exposure_files() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Profile::Pmrapp)
            .await?;
        let efb: &dyn ExposureFileBackend = &backend;
        let eb: &dyn ExposureBackend = &backend;

        let w1 = make_example_workspace(&backend).await?;
        let e1 = make_example_exposure(&backend, w1).await?;
        let ef0 = make_example_exposure_file(
            &backend, e1, "README.md").await?;

        let e2 = make_example_exposure(&backend, w1).await?;
        let ef1 = make_example_exposure_file(&backend, e2, "README.md").await?;
        make_example_exposure_file(&backend, e2, "model.cellml").await?;
        make_example_exposure_file(&backend, e2, "lib/units.cellml").await?;
        let results = efb.list_for_exposure(e2).await?;
        assert_eq!(3, results.len());
        assert_eq!(
            vec!["README.md", "model.cellml", "lib/units.cellml"],
            results.iter()
                .map(|ef| &ef.workspace_file_path)
                .collect::<Vec<_>>(),
        );

        // Matching pairing of exposure id and file
        assert!(eb.set_default_file(e1, ef0).await?);
        assert!(eb.set_default_file(e2, ef1).await?);
        // Mismatching pairing of exposure id and file
        assert!(!eb.set_default_file(e2, ef0).await?);
        assert!(!eb.set_default_file(e1, ef1).await?);

        Ok(())
    }

    #[async_std::test]
    async fn test_exposure_backend() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Profile::Pmrapp)
            .await?;
        let w = make_example_workspace(&backend).await?;
        let id = make_example_exposure(&backend, w).await?;
        make_example_exposure_file(&backend, id, "README.md").await?;
        make_example_exposure_file(&backend, id, "model.cellml").await?;
        make_example_exposure_file(&backend, id, "lib/units.cellml").await?;
        // let exposure = b.get_exposure(id).await?;
        let exposure = Backend::get_exposure(&backend, id).await?;
        // exposure.backend;
        let exposure_files = Backend::get_exposure_files(&backend, exposure.id()).await?;
        assert_eq!(3, exposure_files.len());
        let exposure_files = exposure.files().await?;
        assert_eq!(3, exposure_files.len());
        assert_eq!(
            vec!["README.md", "model.cellml", "lib/units.cellml"],
            exposure_files.iter()
                .map(|ef| ef.workspace_file_path())
                .collect::<Vec<_>>(),
        );

        Ok(())
    }

}
