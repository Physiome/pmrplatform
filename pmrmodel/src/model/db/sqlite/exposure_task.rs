use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    exposure::task::traits::ExposureTaskBackend,
    profile::ViewTaskTemplate,
};

use crate::{
    backend::db::SqliteBackend,
};

async fn insert_exposure_file_view_task_template_sqlite(
    sqlite: &SqliteBackend,
    exposure_file_id: i64,
    view_task_template_id: i64,
) -> Result<i64, BackendError> {
    let id = sqlx::query!(
        r#"
INSERT INTO exposure_file_view_task_template (
    exposure_file_id,
    view_task_template_id
)
VALUES ( ?1, ?2 )
        "#,
        exposure_file_id,
        view_task_template_id,
    )
    .execute(&*sqlite.pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

async fn select_exposure_file_view_task_template_sqlite(
    sqlite: &SqliteBackend,
    exposure_file_id: i64,
) -> Result<Vec<ViewTaskTemplate>, BackendError> {
    let rec = sqlx::query!(
        r#"
SELECT
    id,
    view_key,
    description,
    task_template_id,
    updated_ts
FROM
    view_task_template
WHERE
    id IN (
        SELECT
            view_task_template_id
        FROM
            exposure_file_view_task_template
        WHERE
            exposure_file_id = ?
    )
        "#,
        exposure_file_id,
    )
    .map(|row| ViewTaskTemplate {
        id: row.id,
        view_key: row.view_key,
        description: row.description,
        task_template_id: row.task_template_id,
        updated_ts: row.updated_ts,
        // task_template is from the other backend
        task_template: None,
    })
    .fetch_all(&*sqlite.pool)
    .await?;
    Ok(rec)
}

#[async_trait]
impl ExposureTaskBackend for SqliteBackend {
    async fn set_file_templates(
        &self,
        exposure_file_id: i64,
        mut view_task_template_ids: impl Iterator<Item = i64> + Send,
    ) -> Result<(), BackendError> {
        // TODO delete old templates
        while let Some(view_task_template_id) = view_task_template_ids.next() {
            insert_exposure_file_view_task_template_sqlite(
                &self,
                exposure_file_id,
                view_task_template_id,
            ).await?;
        };
        Ok(())
    }

    async fn get_file_templates(
        &self,
        exposure_file_id: i64,
    ) -> Result<Vec<ViewTaskTemplate>, BackendError> {
        select_exposure_file_view_task_template_sqlite(
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
    use crate::backend::db::{
        Profile::Pmrapp,
        SqliteBackend,
    };

    use crate::model::db::sqlite::{
        exposure::testing::make_example_exposure,
        exposure_file::testing::make_example_exposure_file,
        workspace::testing::make_example_workspace,
    };

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Pmrapp)
            .await?;

        let vttb: &dyn ViewTaskTemplateBackend = &backend;
        let v1 = vttb.insert_view_task_template("view1", "", 1).await?;
        let v2 = vttb.insert_view_task_template("view2", "", 2).await?;
        let v3 = vttb.insert_view_task_template("view3", "", 3).await?;
        let v4 = vttb.insert_view_task_template("view4", "", 4).await?;

        let workspace_id = make_example_workspace(&backend).await?;
        let exposure_id = make_example_exposure(&backend, workspace_id).await?;
        let exposure_file_1 = make_example_exposure_file(
            &backend, exposure_id, "some_demo_file").await?;
        let exposure_file_2 = make_example_exposure_file(
            &backend, exposure_id, "some_other_demo_file").await?;

        ExposureTaskBackend::set_file_templates(
            &backend, exposure_file_1, [v1, v2, v3].into_iter()
        ).await?;
        ExposureTaskBackend::set_file_templates(
            &backend, exposure_file_2, [v2, v4].into_iter()
        ).await?;

        let templates1 = ExposureTaskBackend::get_file_templates(
            &backend, exposure_file_1).await?;
        assert_eq!(templates1.len(), 3);

        let templates2 = ExposureTaskBackend::get_file_templates(
            &backend, exposure_file_2).await?;
        assert_eq!(templates2.len(), 2);

        // TODO include following test for delete

        // ExposureTaskBackend::set_file_templates(
        //     &backend, exposure_file_1, [v2, v4].into_iter()
        // ).await?;
        // let templates1 = ExposureTaskBackend::get_file_templates(
        //     &backend, exposure_file_1).await?;
        // assert_eq!(templates1.len(), 2);

        Ok(())
    }
}
