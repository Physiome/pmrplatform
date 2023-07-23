use async_trait::async_trait;
#[cfg(not(test))]
use chrono::Utc;
#[cfg(test)]
use crate::test::Utc;
use pmrmodel_base::{
    error::BackendError,
    profile::ViewTaskTemplate,
    profile::traits::ViewTaskTemplateBackend,
};

use crate::{
    backend::db::SqliteBackend,
};

async fn insert_view_task_template_sqlite(
    sqlite: &SqliteBackend,
    view_key: &str,
    description: &str,
    task_template_id: i64,
) -> Result<i64, BackendError> {
    let updated_ts = Utc::now().timestamp();
    let id = sqlx::query!(
        r#"
INSERT INTO view_task_template (
    view_key,
    description,
    task_template_id,
    updated_ts
)
VALUES ( ?1, ?2, ?3, ?4 )
        "#,
        view_key,
        description,
        task_template_id,
        updated_ts,
    )
    .execute(&*sqlite.pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

async fn update_view_task_template_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
    view_key: &str,
    description: &str,
    task_template_id: i64,
) -> Result<bool, BackendError> {
    let updated_ts = Utc::now().timestamp();
    let rows_affected = sqlx::query!(
        r#"
UPDATE
    view_task_template
SET
    view_key = ?2,
    description = ?3,
    task_template_id = ?4,
    updated_ts = ?5
WHERE
    id = ?1
        "#,
        id,
        view_key,
        description,
        task_template_id,
        updated_ts,
    )
    .execute(&*sqlite.pool)
    .await?
    .rows_affected();
    Ok(rows_affected > 0)
}

async fn select_view_task_template_by_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<ViewTaskTemplate, BackendError> {
    let result = sqlx::query_as!(
        ViewTaskTemplate,
        r#"
SELECT
    id,
    view_key,
    description,
    task_template_id,
    updated_ts
FROM view_task_template
WHERE id = ?1
        "#,
        id,
    )
    .fetch_one(&*sqlite.pool)
    .await?;
    Ok(result)
}

#[async_trait]
impl ViewTaskTemplateBackend for SqliteBackend {
    async fn insert_view_task_template(
        &self,
        view_key: &str,
        description: &str,
        task_template_id: i64,
    ) -> Result<i64, BackendError> {
        insert_view_task_template_sqlite(
            &self,
            view_key,
            description,
            task_template_id,
        ).await
    }
    async fn update_view_task_template(
        &self,
        id: i64,
        view_key: &str,
        description: &str,
        task_template_id: i64,
    ) -> Result<bool, BackendError> {
        update_view_task_template_sqlite(
            &self,
            id,
            view_key,
            description,
            task_template_id,
        ).await
    }
    async fn select_view_task_template_by_id(
        &self,
        id: i64,
    ) -> Result<ViewTaskTemplate, BackendError> {
        select_view_task_template_by_id_sqlite(&self, id).await
    }
}

#[cfg(test)]
mod testing {
    use pmrmodel_base::profile::{
        ViewTaskTemplate,
        traits::ViewTaskTemplateBackend,
    };
    use crate::backend::db::{
        Profile::Pmrapp,
        SqliteBackend,
    };

    use crate::test::set_timestamp;

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Pmrapp)
            .await?;
        let b: &dyn ViewTaskTemplateBackend = &backend;
        let view_task_template_id = b.insert_view_task_template(
            "test_view",
            "",
            1,
        ).await?;
        let view_task_template = b.select_view_task_template_by_id(view_task_template_id).await?;
        assert_eq!(view_task_template, ViewTaskTemplate {
            id: view_task_template_id,
            view_key: "test_view".to_string(),
            description: "".to_string(),
            task_template_id: 1,
            updated_ts: 1234567890,
        });

        set_timestamp(1357924680);
        assert!(b.update_view_task_template(
            view_task_template_id,
            "final_view",
            "This is a finalized view.",
            2,
        ).await?);
        let view_task_template = b.select_view_task_template_by_id(view_task_template_id).await?;
        assert_eq!(view_task_template, ViewTaskTemplate {
            id: view_task_template_id,
            view_key: "final_view".to_string(),
            description: "This is a finalized view.".to_string(),
            task_template_id: 2,
            updated_ts: 1357924680,
        });
        Ok(())
    }

}
