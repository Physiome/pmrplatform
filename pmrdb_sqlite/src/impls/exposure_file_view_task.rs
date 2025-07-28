use async_trait::async_trait;
use pmrcore::{
    error::{
        BackendError,
        Error,
    },
    exposure::task::{
        ExposureFileViewTask,
        traits::ExposureTaskBackend,
    },
};

use crate::{
    SqliteBackend,
    chrono::Utc,
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

async fn finalize_exposure_file_view_task_with_task_id_sqlite(
    sqlite: &SqliteBackend,
    task_id: i64,
) -> Result<Option<(i64, Option<String>)>, Error> {
    let some_task_id = Some(task_id);
    let mut tx = sqlite.pool.begin().await
        .map_err(BackendError::from)?;
    let rows_affected = sqlx::query!(
        r#"
UPDATE
    exposure_file_view_task
SET
    ready = true
WHERE
    task_id = ?1
        "#,
        some_task_id,
    )
        .execute(&mut *tx)
        .await
        .map_err(BackendError::from)?
        .rows_affected();

    if rows_affected == 0 {
        tx.rollback().await
            .map_err(BackendError::from)?;
        return Err(Error::Backend(BackendError::AppInvariantViolation(
            format!(
                "no exposure_file_view_task bound with task_id: {}",
                task_id,
            )
        )))
    }

    let result = sqlx::query!(
        r#"
UPDATE
    exposure_file_view
SET
    view_key = (
        SELECT
            view_key
        FROM
            view_task_template
        WHERE id = (
            SELECT
                view_task_template_id
            FROM
                exposure_file_view
            WHERE
                exposure_file_view_task_id = (
                    SELECT
                        id
                    FROM
                        exposure_file_view_task
                    WHERE
                        task_id = ?1
                )
        )
    )
WHERE id = (
    SELECT
        id
    FROM
        exposure_file_view
    WHERE
        exposure_file_view_task_id = (
            SELECT
                id
            FROM
                exposure_file_view_task
            WHERE
                task_id = ?1
        )
)
RETURNING id, view_key
"#,
        some_task_id,
    )
    // TODO if the view_key isn't updated it should be an error...
    .fetch_optional(&mut *tx)
    .await
    .map_err(BackendError::from)?
    .map(|result| (result.id, result.view_key));

    tx.commit().await.map_err(BackendError::from)?;

    // ... there are no distinctions between that or the view_key isn't
    // defined, but a None result should indicate that something has
    // gone wrong.
    Ok(result)
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

    async fn finalize_task_id(
        &self,
        task_id: i64,
    ) -> Result<Option<(i64, Option<String>)>, Error> {
        finalize_exposure_file_view_task_with_task_id_sqlite(
            &self,
            task_id,
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pmrcore::profile::traits::ViewTaskTemplateBackend;
    use crate::SqliteBackend;

    use super::super::{
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
        // normally this is done, but we need the inner type not the opaque impl
        // let backend = SqliteBackend::mc("sqlite::memory:".into())
        //     .await
        //     .map_err(anyhow::Error::from_boxed)?;
        let backend = SqliteBackend::connect("sqlite::memory:".into())
            .await?
            .migrate_mc()
            .await?;

        // this is now done by `make_example_exposure_file_view`
        // let vttb: &dyn ViewTaskTemplateBackend = &backend;
        // let v1 = vttb.insert_view_task_template("view1", "", 1).await?;

        let workspace_id = make_example_workspace(&backend).await?;
        let exposure_id = make_example_exposure(&backend, workspace_id).await?;
        let exposure_file_id = make_example_exposure_file(
            &backend, exposure_id, "some_demo_file").await?;
        let (exposure_file_view_id, v1) = make_example_exposure_file_view(
            &backend, exposure_file_id, None, "view1").await?;
        let _ = make_example_exposure_file_view_task(
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
        let _ = make_example_exposure_file_view_task(
            &backend, exposure_file_view_id, v1, None).await?;
        let et = etb.select_task_for_view(exposure_file_view_id).await?;
        assert_eq!(et.map(|et| et.id), Some(2));

        let _ = make_example_exposure_file_view_task(
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
        // normally this is done, but we need the inner type not the opaque impl
        // let backend = SqliteBackend::mc("sqlite::memory:".into())
        //     .await
        //     .map_err(anyhow::Error::from_boxed)?;
        let backend = SqliteBackend::connect("sqlite::memory:".into())
            .await?
            .migrate_mc()
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
