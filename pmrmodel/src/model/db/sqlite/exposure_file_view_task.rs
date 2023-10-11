use async_trait::async_trait;
#[cfg(not(test))]
use chrono::Utc;
#[cfg(test)]
use crate::test::Utc;
use pmrcore::{
    error::BackendError,
    exposure::task::{
        ExposureFileViewTask,
        traits::ExposureTaskBackend,
    },
    profile::ViewTaskTemplate,
};

use crate::{
    backend::db::SqliteBackend,
};

async fn insert_exposure_file_view_task_sqlite(
    sqlite: &SqliteBackend,
    exposure_file_view_id: i64,
    view_task_template_id: i64,
    task_id: Option<i64>,
) -> Result<i64, BackendError> {
    let mut tx = sqlite.pool.begin().await?;
    let created_ts = task_id.map(|_| Utc::now().timestamp());
    let id = sqlx::query!(
        r#"
INSERT INTO exposure_file_view_task (
    exposure_file_view_id,
    view_task_template_id,
    task_id,
    created_ts,
    ready
)
VALUES ( ?1, ?2, ?3, ?4, false )
        "#,
        exposure_file_view_id,
        view_task_template_id,
        task_id,
        created_ts,
    )
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    let rows_affected = sqlx::query!(
        r#"
UPDATE exposure_file_view
SET
    exposure_file_view_task_id = ?1
WHERE
    id = ?2
        "#,
        id,
        exposure_file_view_id,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        tx.rollback().await?;
        Err(BackendError::AppInvariantViolation(
            format!(
                "no such exposure_file_view with id: {}",
                exposure_file_view_id,
            )
        ))
    } else {
        tx.commit().await?;
        Ok(id)
    }
}

async fn select_exposure_file_view_task_for_view_sqlite(
    sqlite: &SqliteBackend,
    exposure_file_view_id: i64,
) -> Result<Option<ExposureFileViewTask>, BackendError> {
    let rec = sqlx::query_as!(
        ExposureFileViewTask,
        r#"
SELECT
    id,
    exposure_file_view_id,
    view_task_template_id,
    task_id,
    created_ts,
    ready
FROM
    exposure_file_view_task
WHERE
    id = (
        SELECT
            exposure_file_view_task_id
        FROM
            exposure_file_view
        WHERE
            exposure_file_view_id = ?
    )
        "#,
        exposure_file_view_id,
    )
    .fetch_one(&*sqlite.pool)
    .await
    .ok();
    Ok(rec)
}

#[async_trait]
impl ExposureTaskBackend for SqliteBackend {
    async fn create_task_for_view(
        &self,
        exposure_file_view_id: i64,
        view_task_template_id: i64,
        task_id: Option<i64>,
    ) -> Result<i64, BackendError> {
        insert_exposure_file_view_task_sqlite(
            &self,
            exposure_file_view_id,
            view_task_template_id,
            task_id,
        ).await
    }

    async fn select_task_for_view(
        &self,
        exposure_file_id: i64,
    ) -> Result<Option<ExposureFileViewTask>, BackendError> {
        select_exposure_file_view_task_for_view_sqlite(
            &self,
            exposure_file_id,
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pmrcore::profile::{
        ViewTaskTemplate,
        traits::ViewTaskTemplateBackend,
    };
    use sqlx::Row;
    use crate::backend::db::{
        Profile::Pmrapp,
        SqliteBackend,
    };

    use crate::model::db::sqlite::{
        exposure::testing::make_example_exposure,
        exposure_file::testing::make_example_exposure_file,
        exposure_file_view::testing::make_example_exposure_file_view,
        workspace::testing::make_example_workspace,
    };

    pub(crate) async fn make_example_exposure_file_view_task(
        backend: &dyn ExposureTaskBackend,
        exposure_file_view_id: i64,
        view_task_template_id: i64,
        task_id: Option<i64>,
    ) -> anyhow::Result<i64> {
        Ok(backend.create_task_for_view(
            exposure_file_view_id,
            view_task_template_id,
            task_id,
        ).await?)
    }

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Pmrapp)
            .await?;

        let vttb: &dyn ViewTaskTemplateBackend = &backend;
        let v1 = vttb.insert_view_task_template("view1", "", 1).await?;

        let workspace_id = make_example_workspace(&backend).await?;
        let exposure_id = make_example_exposure(&backend, workspace_id).await?;
        let exposure_file_id = make_example_exposure_file(
            &backend, exposure_id, "some_demo_file").await?;
        let exposure_file_view_id = make_example_exposure_file_view(
            &backend, exposure_file_id, None, None).await?;
        let exposure_file_view_task_id = make_example_exposure_file_view_task(
            &backend, exposure_file_view_id, v1, None).await?;

        let etb: &dyn ExposureTaskBackend = &backend;
        let et = etb.select_task_for_view(exposure_file_view_id).await?;
        assert_eq!(et, Some(ExposureFileViewTask {
            id: 1,
            exposure_file_view_id: 1,
            view_task_template_id: 1,
            task_id: None,
            created_ts: None,
            ready: false,
        }));

        // add a few more tasks
        let exposure_file_view_task_id = make_example_exposure_file_view_task(
            &backend, exposure_file_view_id, v1, None).await?;
        let et = etb.select_task_for_view(exposure_file_view_id).await?;
        assert_eq!(et.map(|et| et.id), Some(2));

        let exposure_file_view_task_id = make_example_exposure_file_view_task(
            &backend, exposure_file_view_id, v1, None).await?;
        let et = etb.select_task_for_view(exposure_file_view_id).await?;
        assert_eq!(et.map(|et| et.id), Some(3));

        let c = sqlx::query!(
            "SELECT COUNT(id) AS count FROM exposure_file_view_task"
        )
            .fetch_one(&*backend.pool)
            .await?;
        assert_eq!(c.count, 3);

        Ok(())
    }

    #[async_std::test]
    async fn test_failure() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Pmrapp)
            .await?;

        let vttb: &dyn ViewTaskTemplateBackend = &backend;
        let v1 = vttb.insert_view_task_template("view1", "", 1).await?;
        let exposure_file_view_task_id = make_example_exposure_file_view_task(
            &backend, 1, v1, None).await;
        assert!(exposure_file_view_task_id.is_err());

        // ensure no actual insertion happened.
        let c = sqlx::query!(
            "SELECT COUNT(id) AS count FROM exposure_file_view_task"
        )
            .fetch_one(&*backend.pool)
            .await?;
        assert_eq!(c.count, 0);

        Ok(())
    }

}
