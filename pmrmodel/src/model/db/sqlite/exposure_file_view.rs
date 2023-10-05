use async_trait::async_trait;
#[cfg(not(test))]
use chrono::Utc;
#[cfg(test)]
use crate::test::Utc;
use pmrcore::{
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
    exposure_file_view_task_id: Option<i64>,
) -> Result<i64, BackendError> {
    let updated_ts = Utc::now().timestamp();
    let id = sqlx::query!(
        r#"
INSERT INTO exposure_file_view (
    exposure_file_id,
    exposure_file_view_task_id,
    updated_ts
)
VALUES ( ?1, ?2, ?3 )
        "#,
        exposure_file_id,
        exposure_file_view_task_id,
        updated_ts,
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
    exposure_file_view_task_id,
    view_key,
    updated_ts
FROM exposure_file_view
WHERE id = ?1
"#,
        id,
    )
    .map(|row| ExposureFileView {
        id: row.id,
        exposure_file_id: row.exposure_file_id,
        exposure_file_view_task_id: row.exposure_file_view_task_id,
        view_key: row.view_key,
        updated_ts: row.updated_ts,
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
    exposure_file_view_task_id,
    view_key,
    updated_ts
FROM exposure_file_view
WHERE exposure_file_id = ?1
"#,
        exposure_file_id,
    )
    .map(|row| ExposureFileView {
        id: row.id,
        exposure_file_id: row.exposure_file_id,
        exposure_file_view_task_id: row.exposure_file_view_task_id,
        view_key: row.view_key,
        updated_ts: row.updated_ts,
    })
    .fetch_all(&*sqlite.pool)
    .await?;
    Ok(rec.into())
}

// TODO when the full framework for dealing with tasks are done, the
// view_key will be derived automatically rather than set like so, but
// whether this backdoor need to be removed will be TBD.
async fn update_exposure_file_view_key_by_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
    view_key: &str,
) -> Result<bool, BackendError> {
    let rows_affected = sqlx::query!(r#"
UPDATE exposure_file_view
SET
    view_key = ?2
WHERE id = ?1
"#,
        id,
        view_key,
    )
    .execute(&*sqlite.pool)
    .await?
    .rows_affected();
    Ok(rows_affected > 0)
}

#[async_trait]
impl ExposureFileViewBackend for SqliteBackend {
    async fn insert(
        &self,
        exposure_file_id: i64,
        exposure_file_view_task_id: Option<i64>,
    ) -> Result<i64, BackendError>{
        insert_exposure_file_view_sqlite(
            &self,
            exposure_file_id,
            exposure_file_view_task_id,
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

    async fn update_view_key(
        &self,
        id: i64,
        view_key: &str,
    ) -> Result<bool, BackendError> {
        update_exposure_file_view_key_by_id_sqlite(
            &self,
            id,
            view_key,
        ).await
    }
}

// TODO generalize the testing modules across related modules (actually
// all db access) and instantiate the test of all db implementations
// against all relevant tests.
#[cfg(test)]
pub(crate) mod testing {
    use pmrcore::{
        exposure::{
            ExposureFileView,
            traits::ExposureFile as _,
            traits::ExposureFileBackend,
            traits::ExposureFileView as _,
            traits::ExposureFileViewBackend,
        },
        platform::MCPlatform,
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
        exposure_file_view_task_id: Option<i64>,
        view_key: &str,
    ) -> anyhow::Result<i64> {
        let id = backend.insert(
            exposure_file_id,
            exposure_file_view_task_id,
        ).await?;
        backend.update_view_key(id, view_key).await?;
        Ok(id)
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
            &backend, exposure_file_id, None, "some_view",
        ).await?;
        let exposure_file_view = efvb.get_id(id).await?;
        assert_eq!(exposure_file_view, ExposureFileView {
            id: 1,
            exposure_file_view_task_id: None,
            exposure_file_id: 1,
            view_key: Some("some_view".into()),
            updated_ts: 1234567890,
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
            &backend, e2f1, None, "view").await?;

        let e2f2 = make_example_exposure_file(
            &backend, e2, "model.cellml").await?;
        make_example_exposure_file_view(&backend, e2f2, None, "model").await?;
        make_example_exposure_file_view(&backend, e2f2, None, "math").await?;
        make_example_exposure_file_view(&backend, e2f2, None, "code").await?;
        make_example_exposure_file_view(&backend, e2f2, None, "sim").await?;
        let results = efvb.list_for_exposure_file(e2f2).await?;
        assert_eq!(4, results.len());
        let mut views = results.iter()
            .map(|efv| (efv.id, efv.view_key.as_ref().map(|x| x.as_ref())))
            .collect::<Vec<_>>();
        views.sort();
        assert_eq!(
            vec![
                (2, Some("model")),
                (3, Some("math")),
                (4, Some("code")),
                (5, Some("sim")),
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

        let v = MCPlatform::get_exposure_file_view(&backend, 2).await?;
        assert_eq!(v.view_key(), Some("model"));
        assert_eq!(v.exposure_file().await?.id(), e2);

        Ok(())
    }

}
