use async_trait::async_trait;
use chrono::Utc;
use pmrmodel_base::task_template::{
    TaskTemplate,
    TaskTemplateArg,
    TaskTemplateArgChoice,
};
use textwrap_macros::dedent;

use crate::backend::db::SqliteBackend;

async fn add_task_template_sqlite(
    sqlite: &SqliteBackend,
    bin_path: String,
    version_id: String,
) -> anyhow::Result<i64> {
    let created_ts = Utc::now().timestamp();

    let id = sqlx::query!(
        r#"
INSERT INTO task_template (
    bin_path,
    version_id,
    created_ts
)
VALUES ( ?1, ?2, ?3 )
        "#,
        bin_path,
        version_id,
        created_ts,
    )
    .execute(&*sqlite.pool)
    .await?
    .last_insert_rowid();

    Ok(id)
}

async fn get_task_template_by_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> anyhow::Result<TaskTemplate> {
    let rec = sqlx::query_as(
        dedent!(r#"
        SELECT
            id,
            bin_path,
            version_id,
            created_ts,
            final_task_template_arg_id,
            superceded_by_id
        FROM task_template
        WHERE id = ?1
        "#),
    )
    .bind(id)
    .fetch_one(&*sqlite.pool)
    .await?;
    Ok(rec)
}

async fn add_task_template_arg_sqlite(
    sqlite: &SqliteBackend,
    task_template_id: i64,
    flag: Option<String>,
    flag_joined: bool,
    prompt: Option<String>,
    default_value: Option<String>,
    choice_fixed: bool,
    choice_source: Option<String>,
) -> anyhow::Result<i64> {
    let id = sqlx::query!(
        r#"
INSERT INTO task_template_arg (
    task_template_id,
    flag,
    flag_joined,
    prompt,
    default_value,
    choice_fixed,
    choice_source
)
VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7 )
        "#,
        task_template_id,
        flag,
        flag_joined,
        prompt,
        default_value,
        choice_fixed,
        choice_source,
    )
    .execute(&*sqlite.pool)
    .await?
    .last_insert_rowid();

    Ok(id)
}

async fn add_task_template_arg_choice_sqlite(
    sqlite: &SqliteBackend,
    task_template_arg_id: i64,
    value: Option<String>,
    label: String,
) -> anyhow::Result<i64> {
    let id = sqlx::query!(
        r#"
INSERT INTO task_template_arg_choice (
    task_template_arg_id,
    value,
    label
)
VALUES ( ?1, ?2, ?3 )
        "#,
        task_template_arg_id,
        value,
        label,
    )
    .execute(&*sqlite.pool)
    .await?
    .last_insert_rowid();

    Ok(id)
}

#[async_trait]
pub trait TaskTemplateBackend {
    async fn add_task_template(
        &self,
        bin_path: String,
        version_id: String,
    ) -> anyhow::Result<i64>;
    async fn get_task_template_by_id(
        &self,
        id: i64,
    ) -> anyhow::Result<TaskTemplate>;
}

#[async_trait]
impl TaskTemplateBackend for SqliteBackend {
    async fn add_task_template(
        &self,
        bin_path: String,
        version_id: String,
    ) -> anyhow::Result<i64> {
        add_task_template_sqlite(&self, bin_path, version_id).await
    }

    async fn get_task_template_by_id(
        &self,
        id: i64,
    ) -> anyhow::Result<TaskTemplate> {
        get_task_template_by_id_sqlite(&self, id).await
    }
}
