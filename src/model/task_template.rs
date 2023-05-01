use futures::future;
use async_trait::async_trait;
#[cfg(not(test))]
use chrono::Utc;
use pmrmodel_base::task_template::{
    TaskTemplate,
    TaskTemplateArg,
    TaskTemplateArgChoice,
};
use textwrap_macros::dedent;

use crate::backend::db::SqliteBackend;

#[cfg(test)]
use crate::test::Utc;

async fn add_task_template_sqlite(
    sqlite: &SqliteBackend,
    bin_path: &str,
    version_id: &str,
) -> Result<i64, sqlx::Error> {
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

async fn finalize_task_template_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query!(
        r#"
UPDATE task_template
SET
    final_task_template_arg_id = (
        SELECT COALESCE(
            (
                SELECT
                    final_task_template_arg_id
                FROM task_template_arg
                WHERE id = ?1
            ),
            MAX(id), 0
        )
        FROM task_template_arg
        WHERE task_template_id = ?1
    )
WHERE id = ?1
        "#,
        id,
    )
    .execute(&*sqlite.pool)
    .await?;

    Ok(id)
}

async fn get_task_template_by_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<TaskTemplate, sqlx::Error> {
    let rec = sqlx::query!(r#"
SELECT
    id,
    bin_path,
    version_id,
    created_ts,
    final_task_template_arg_id,
    superceded_by_id
FROM task_template
WHERE id = ?1
"#,
        id,
    )
    .map(|row| TaskTemplate {
        id: row.id,
        bin_path: row.bin_path,
        version_id: row.version_id,
        created_ts: row.created_ts,
        final_task_template_arg_id: row.final_task_template_arg_id,
        superceded_by_id: row.superceded_by_id,
        args: None,
    })
    .fetch_one(&*sqlite.pool)
    .await?;
    Ok(rec)
}

async fn add_task_template_arg_sqlite(
    sqlite: &SqliteBackend,
    task_template_id: i64,
    flag: Option<&str>,
    flag_joined: bool,
    prompt: Option<&str>,
    default_value: Option<&str>,
    choice_fixed: bool,
    choice_source: Option<&str>,
) -> Result<i64, sqlx::Error> {
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
    value: Option<&str>,
    label: &str,
) -> Result<i64, sqlx::Error> {
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

async fn get_task_template_args_by_task_template_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<Vec<TaskTemplateArg>, sqlx::Error> {
    let rec = sqlx::query!(r#"
SELECT
    id,
    task_template_id,
    flag,
    flag_joined,
    prompt,
    default_value,
    choice_fixed,
    choice_source
FROM task_template_arg
WHERE task_template_id = ?1 AND id <= (
    SELECT final_task_template_arg_id
    FROM task_template
    WHERE id = ?1
)
"#,
        id,
    )
    .map(|row| TaskTemplateArg {
        id: row.id,
        task_template_id: row.task_template_id,
        flag: row.flag,
        flag_joined: row.flag_joined,
        prompt: row.prompt,
        default_value: row.default_value,
        choice_fixed: row.choice_fixed,
        choice_source: row.choice_source,
        choices: None,
    })
    .fetch_all(&*sqlite.pool)
    .await?;
    Ok(rec)
}

#[async_trait]
pub trait TaskTemplateBackend {
    // add a new task template that's open to updates
    async fn add_new_task_template(
        &self,
        bin_path: &str,
        version_id: &str,
    ) -> Result<i64, sqlx::Error>;
    // adds a completed task template.
    async fn add_task_template(
        &self,
        bin_path: &str,
        version_id: &str,
        arguments: &[(
            Option<&str>,
            bool,
            Option<&str>,
            Option<&str>,
            bool,
            Option<&str>,
        )],
    ) -> Result<i64, sqlx::Error>;
    // finalize an open task template.
    async fn finalize_new_task_template(
        &self,
        id: i64,
    ) -> Result<i64, sqlx::Error>;
    async fn add_task_template_arg_choice(
        &self,
        task_template_arg_id: i64,
        value: Option<&str>,
        label: &str,
    ) -> Result<i64, sqlx::Error>;
    async fn get_task_template_by_id(
        &self,
        id: i64,
    ) -> Result<TaskTemplate, sqlx::Error>;
}

#[async_trait]
impl TaskTemplateBackend for SqliteBackend {
    async fn add_new_task_template(
        &self,
        bin_path: &str,
        version_id: &str,
    ) -> Result<i64, sqlx::Error> {
        add_task_template_sqlite(&self, bin_path, version_id).await
    }

    async fn finalize_new_task_template(
        &self,
        id: i64,
    ) -> Result<i64, sqlx::Error> {
        finalize_task_template_sqlite(&self, id).await
    }

    async fn add_task_template_arg_choice(
        &self,
        task_template_arg_id: i64,
        value: Option<&str>,
        label: &str,
    ) -> Result<i64, sqlx::Error> {
        add_task_template_arg_choice_sqlite(
            &self,
            task_template_arg_id,
            value,
            label,
        ).await
    }

    async fn add_task_template(
        &self,
        bin_path: &str,
        version_id: &str,
        arguments: &[(
            Option<&str>,
            bool,
            Option<&str>,
            Option<&str>,
            bool,
            Option<&str>,
        )],
    ) -> Result<i64, sqlx::Error> {
        let result = add_task_template_sqlite(&self, bin_path, version_id).await?;
        let mut tasks = arguments.into_iter()
            .map(|x| { add_task_template_arg_sqlite(
                &self,
                result,
                x.0,
                x.1,
                x.2,
                x.3,
                x.4,
                x.5,
            )})
            .into_iter();
        // execute sequentially to ensure insertion order
        while let Some(task) = tasks.next() {
            task.await?;
        }
        finalize_task_template_sqlite(&self, result).await?;
        Ok(result)
    }

    async fn get_task_template_by_id(
        &self,
        id: i64,
    ) -> Result<TaskTemplate, sqlx::Error> {
        let mut result = get_task_template_by_id_sqlite(&self, id).await?;
        result.args = Some(get_task_template_args_by_task_template_id_sqlite(
            &self, result.id
        ).await?);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use pmrmodel_base::task_template::{
        TaskTemplate,
        TaskTemplateArg,
    };
    use crate::backend::db::{
        Profile,
        SqliteBackend,
    };
    use crate::model::task_template::TaskTemplateBackend;

    #[async_std::test]
    async fn test_smoketest_no_args() {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await
            .unwrap()
            .run_migration_profile(Profile::Pmrtqs)
            .await
            .unwrap();

        let id = TaskTemplateBackend::add_task_template(
            &backend, "/bin/true", "1.0.0", &[],
        ).await
            .unwrap();
        let template = TaskTemplateBackend::get_task_template_by_id(
            &backend, id
        ).await
            .unwrap();
        assert_eq!(template, TaskTemplate {
            id: 1,
            bin_path: "/bin/true".into(),
            version_id: "1.0.0".into(),
            created_ts: 1234567890,
            final_task_template_arg_id: Some(0),
            superceded_by_id: None,
            args: Some([].to_vec()),
        });
    }

    #[async_std::test]
    async fn test_smoketest_with_args() {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await
            .unwrap()
            .run_migration_profile(Profile::Pmrtqs)
            .await
            .unwrap();

        let id = TaskTemplateBackend::add_task_template(
            &backend, "/bin/echo", "1.0.0", &[(
                None,
                false,
                Some("First statement"),
                None,
                false,
                None,
            ), (
                None,
                false,
                Some("Second statement"),
                None,
                false,
                None,
            )],
        ).await
            .unwrap();
        let template = TaskTemplateBackend::get_task_template_by_id(
            &backend, id
        ).await
            .unwrap();
        let answer = TaskTemplate {
            id: 1,
            bin_path: "/bin/echo".into(),
            version_id: "1.0.0".into(),
            created_ts: template.created_ts,  // matching itself
            final_task_template_arg_id: Some(2),
            superceded_by_id: None,
            args: Some([TaskTemplateArg {
                id: 1,
                task_template_id: 1,
                flag: None,
                flag_joined: false,
                prompt: Some("First statement".into()),
                default_value: None,
                choice_fixed: false,
                choice_source: None,
                choices: None
            }, TaskTemplateArg {
                id: 2,
                task_template_id: 1,
                flag: None,
                flag_joined: false,
                prompt: Some("Second statement".into()),
                default_value: None,
                choice_fixed: false,
                choice_source: None,
                choices: None
            }].to_vec()),
        };
        assert_eq!(template, answer);
        assert_eq!(template, serde_json::from_str(r#"
        {
            "id": 1,
            "bin_path": "/bin/echo",
            "version_id": "1.0.0",
            "created_ts": 1234567890,
            "final_task_template_arg_id": 2,
            "superceded_by_id": null,
            "args": [
                {
                    "id": 1,
                    "task_template_id": 1,
                    "flag": null,
                    "flag_joined": false,
                    "prompt": "First statement",
                    "default_value": null,
                    "choice_fixed": false,
                    "choice_source": null,
                    "choices": null
                },
                {
                    "id": 2,
                    "task_template_id": 1,
                    "flag": null,
                    "flag_joined": false,
                    "prompt": "Second statement",
                    "default_value": null,
                    "choice_fixed": false,
                    "choice_source": null,
                    "choices": null
                }
            ]
        }
        "#).unwrap());
    }

    #[async_std::test]
    async fn test_add_manual_finalize() {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await
            .unwrap()
            .run_migration_profile(Profile::Pmrtqs)
            .await
            .unwrap();

        let id = TaskTemplateBackend::add_new_task_template(
            &backend, "/bin/true", "1.0.0",
        ).await
            .unwrap();
        // not yet finalized
        let template = TaskTemplateBackend::get_task_template_by_id(
            &backend, id
        ).await.unwrap();
        assert_eq!(template, TaskTemplate {
            id: 1,
            bin_path: "/bin/true".into(),
            version_id: "1.0.0".into(),
            created_ts: 1234567890,
            final_task_template_arg_id: None,
            superceded_by_id: None,
            args: Some([].to_vec()),
        });

        // finalizing
        TaskTemplateBackend::finalize_new_task_template(
            &backend, id,
        ).await.unwrap();

        // should reflect finalized id of 0
        let template = TaskTemplateBackend::get_task_template_by_id(
            &backend, id
        ).await.unwrap();
        assert_eq!(template, TaskTemplate {
            id: 1,
            bin_path: "/bin/true".into(),
            version_id: "1.0.0".into(),
            created_ts: 1234567890,
            final_task_template_arg_id: Some(0),
            superceded_by_id: None,
            args: Some([].to_vec()),
        });

        // doing a manual insert to avoid API changes that may prevent this
        sqlx::query(
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
            VALUES ( 1, "-h", FALSE, "The prompt", NULL, FALSE, NULL)
            "#,
        )
        .execute(&*backend.pool)
        .await
        .unwrap();

        // attempt to finalize again despite choice manually added
        TaskTemplateBackend::finalize_new_task_template(
            &backend, id,
        ).await.unwrap();
        let template = TaskTemplateBackend::get_task_template_by_id(
            &backend, id
        ).await.unwrap();

        // can't be added.
        assert_eq!(template, TaskTemplate {
            id: 1,
            bin_path: "/bin/true".into(),
            version_id: "1.0.0".into(),
            created_ts: 1234567890,
            final_task_template_arg_id: Some(0),
            superceded_by_id: None,
            args: Some([].to_vec()),
        });

    }

}
