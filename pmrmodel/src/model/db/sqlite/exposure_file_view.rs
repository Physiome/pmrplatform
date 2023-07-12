use async_trait::async_trait;
use pmrmodel_base::{
    error::BackendError,
    exposure::{
        ExposureFileView,
        ExposureFileViews,
        traits::ExposureFileViewBackend,
    },
};

use crate::{
    backend::db::SqliteBackend,
};

async fn insert_exposure_file_view_sqlite(
    sqlite: &SqliteBackend,
    exposure_file_id: i64,
    view_key: &str,
) -> Result<i64, BackendError> {
    let id = sqlx::query!(
        r#"
INSERT INTO exposure_file_view (
    exposure_file_id,
    view_key
)
VALUES ( ?1, ?2 )
        "#,
        exposure_file_id,
        view_key,
    )
    .execute(&*sqlite.pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

async fn get_exposure_file_view_by_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<ExposureFileView, BackendError> {
    let rec = sqlx::query!(r#"
SELECT
    id,
    exposure_file_id,
    view_key
FROM exposure_file_view
WHERE id = ?1
"#,
        id,
    )
    .map(|row| ExposureFileView {
        id: row.id,
        exposure_file_id: row.exposure_file_id,
        view_key: row.view_key,
    })
    .fetch_one(&*sqlite.pool)
    .await?;
    Ok(rec)
}

async fn list_exposure_file_views_for_exposure_file_sqlite(
    sqlite: &SqliteBackend,
    exposure_file_id: i64,
) -> Result<ExposureFileViews, BackendError> {
    let rec = sqlx::query!(r#"
SELECT
    id,
    exposure_file_id,
    view_key
FROM exposure_file_view
WHERE exposure_file_id = ?1
"#,
        exposure_file_id,
    )
    .map(|row| ExposureFileView {
        id: row.id,
        exposure_file_id: row.exposure_file_id,
        view_key: row.view_key,
    })
    .fetch_all(&*sqlite.pool)
    .await?;
    Ok(rec.into())
}

#[async_trait]
impl ExposureFileViewBackend for SqliteBackend {
    async fn insert(
        &self,
        exposure_file_id: i64,
        view_key: &str,
    ) -> Result<i64, BackendError>{
        insert_exposure_file_view_sqlite(
            &self,
            exposure_file_id,
            view_key,
        ).await
    }

    async fn list_for_exposure_file(
        &self,
        exposure_file_id: i64,
    ) -> Result<ExposureFileViews, BackendError> {
        list_exposure_file_views_for_exposure_file_sqlite(
            &self,
            exposure_file_id,
        ).await
    }

    async fn get_id(
        &self,
        id: i64,
    ) -> Result<ExposureFileView, BackendError> {
        get_exposure_file_view_by_id_sqlite(
            &self,
            id,
        ).await
    }
}

// TODO generalize the testing modules across related modules (actually
// all db access) and instantiate the test of all db implementations
// against all relevant tests.
#[cfg(test)]
pub(crate) mod testing {
    use pmrmodel_base::{
        exposure::{
            ExposureFileView,
            traits::ExposureFileBackend,
            traits::ExposureFileViewBackend,
        },
    };
    use crate::backend::db::{
        Profile,
        SqliteBackend,
    };
    use crate::model::db::sqlite::{
        workspace::testing::make_example_workspace,
        exposure::testing::make_example_exposure,
        exposure_file::testing::make_example_exposure_file,
    };

    pub(crate) async fn make_example_exposure_file_view(
        backend: &dyn ExposureFileViewBackend,
        exposure_file_id: i64,
        view_key: &str,
    ) -> anyhow::Result<i64> {
        Ok(backend.insert(
            exposure_file_id,
            view_key,
        ).await?)
    }

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Profile::Pmrapp)
            .await?;
        let efvb: &dyn ExposureFileViewBackend = &backend;

        let exposure_file_id = make_example_exposure_file(
            &backend,
            make_example_exposure(
                &backend,
                make_example_workspace(&backend).await?,
            ).await?,
            "README.md"
        ).await?;
        let id = make_example_exposure_file_view(
            &backend, exposure_file_id, "some_view",
        ).await?;
        let exposure_file_view = efvb.get_id(id).await?;
        assert_eq!(exposure_file_view, ExposureFileView {
            id: 1,
            exposure_file_id: 1,
            view_key: "some_view".into(),
        });
        Ok(())
    }

    #[async_std::test]
    async fn test_using_exposure_file_view() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Profile::Pmrapp)
            .await?;
        let efvb: &dyn ExposureFileViewBackend = &backend;
        let efb: &dyn ExposureFileBackend = &backend;

        let w1 = make_example_workspace(&backend).await?;
        let _ = make_example_exposure(&backend, w1).await?;
        let e2 = make_example_exposure(&backend, w1).await?;
        let e2f1 = make_example_exposure_file(&backend, e2, "README.md").await?;
        let e2f1v1 = make_example_exposure_file_view(
            &backend, e2f1, "view").await?;

        let e2f2 = make_example_exposure_file(
            &backend, e2, "model.cellml").await?;
        make_example_exposure_file_view(&backend, e2f2, "model").await?;
        make_example_exposure_file_view(&backend, e2f2, "math").await?;
        make_example_exposure_file_view(&backend, e2f2, "code").await?;
        make_example_exposure_file_view(&backend, e2f2, "sim").await?;
        let results = efvb.list_for_exposure_file(e2f2).await?;
        assert_eq!(4, results.len());
        let mut views = results.iter()
            .map(|efv| (efv.id, efv.view_key.as_ref()))
            .collect::<Vec<_>>();
        views.sort();
        assert_eq!(
            vec![
                (2, "model"),
                (3, "math"),
                (4, "code"),
                (5, "sim"),
            ],
            views,
        );

        // Matching pairing of exposure file and view
        assert!(efb.set_default_view(e2f1, e2f1v1).await?);
        assert!(efb.set_default_view(e2f2, 2).await?);
        assert!(efb.set_default_view(e2f2, 3).await?);
        // Mismatching pairing of exposure file and view
        assert!(!efb.set_default_view(e2f1, 2).await?);
        assert!(!efb.set_default_view(e2f2, e2f1v1).await?);

        Ok(())
    }

}
