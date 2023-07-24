use async_trait::async_trait;
#[cfg(not(test))]
use chrono::Utc;
use futures::future;
use pmrmodel_base::{
    error::BackendError,
    task_template::{
        TaskTemplate,
        TaskTemplateArg,
        TaskTemplateArgs,
        TaskTemplateArgChoice,
        TaskTemplateArgChoices,
        traits::TaskTemplateBackend,
    },
};

use crate::{
    backend::db::SqliteBackend,
};

#[cfg(test)]
use crate::test::Utc;

async fn add_task_template_sqlite(
    sqlite: &SqliteBackend,
    bin_path: &str,
    version_id: &str,
) -> Result<i64, BackendError> {
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
) -> Result<i64, BackendError> {
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
) -> Result<TaskTemplate, BackendError> {
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

async fn get_task_template_by_arg_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<TaskTemplate, BackendError> {
    let rec = sqlx::query!(r#"
SELECT
    id,
    bin_path,
    version_id,
    created_ts,
    final_task_template_arg_id,
    superceded_by_id
FROM task_template
WHERE id = (
    SELECT task_template_id
    FROM task_template_arg
        WHERE id = ?1
)
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
    default: Option<&str>,
    choice_fixed: bool,
    choice_source: Option<&str>,
) -> Result<i64, BackendError> {
    let id = sqlx::query!(
        r#"
INSERT INTO task_template_arg (
    task_template_id,
    flag,
    flag_joined,
    prompt,
    'default',
    choice_fixed,
    choice_source
)
VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7 )
        "#,
        task_template_id,
        flag,
        flag_joined,
        prompt,
        default,
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
    to_arg: Option<&str>,
    label: &str,
) -> Result<i64, BackendError> {
    let id = sqlx::query!(
        r#"
INSERT INTO task_template_arg_choice (
    task_template_arg_id,
    to_arg,
    label
)
VALUES ( ?1, ?2, ?3 )
        "#,
        task_template_arg_id,
        to_arg,
        label,
    )
    .execute(&*sqlite.pool)
    .await?
    .last_insert_rowid();

    Ok(id)
}

async fn delete_task_template_arg_choice_by_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<Option<TaskTemplateArgChoice>, BackendError> {
    let mut tx = sqlite.pool.begin().await?;
    let rec = sqlx::query_as!(TaskTemplateArgChoice, r#"
DELETE FROM
    task_template_arg_choice
WHERE
    id = ?1
    RETURNING *
"#,
        id,
    )
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(rec)
}

async fn get_task_template_args_by_task_template_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<TaskTemplateArgs, BackendError> {
    let rec = sqlx::query!(r#"
SELECT
    id,
    task_template_id,
    flag,
    flag_joined,
    prompt,
    "default",
    choice_fixed,
    choice_source
FROM task_template_arg,
    (
        SELECT final_task_template_arg_id
        FROM task_template
        WHERE id = ?1
    ) tt
WHERE
    task_template_id = ?1 AND
    (
        tt.final_task_template_arg_id IS NULL OR
        id <= tt.final_task_template_arg_id
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
        default: row.default,
        choice_fixed: row.choice_fixed,
        choice_source: row.choice_source,
        choices: None,
    })
    .fetch_all(&*sqlite.pool)
    .await?;
    Ok(rec.into())
}

async fn get_task_template_arg_by_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
    complete: bool,
) -> Result<Option<TaskTemplateArg>, BackendError> {
    let mut rec = sqlx::query!(r#"
SELECT
    id,
    task_template_id,
    flag,
    flag_joined,
    prompt,
    "default",
    choice_fixed,
    choice_source
FROM
    task_template_arg
WHERE
    id = ?1
"#,
        id,
    )
    .map(|row| TaskTemplateArg {
        id: row.id,
        task_template_id: row.task_template_id,
        flag: row.flag,
        flag_joined: row.flag_joined,
        prompt: row.prompt,
        default: row.default,
        choice_fixed: row.choice_fixed,
        choice_source: row.choice_source,
        choices: None,
    })
    .fetch_optional(&*sqlite.pool)
    .await?;

    match &mut rec {
        Some(ref mut rec) =>
            rec.choices = if complete {
                Some(
                    get_task_template_arg_choices_by_task_template_arg_id_sqlite(
                        sqlite,
                        rec.id,
                    ).await?
                )
            }
            else {
                None
            },
        None => {}
    }

    Ok(rec)
}

async fn delete_task_template_arg_by_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<Option<TaskTemplateArg>, BackendError> {
    let mut tx = sqlite.pool.begin().await?;
    let rec = sqlx::query!(r#"
DELETE FROM
    task_template_arg
WHERE
    (
        SELECT final_task_template_arg_id
        FROM task_template
        WHERE id = (
            SELECT task_template_id
            FROM task_template_arg
            WHERE id = ?1
        )
    ) is NULL AND
    id = ?1
    RETURNING *
"#,
        id,
    )
    .map(|row| TaskTemplateArg {
        id: row.id,
        task_template_id: row.task_template_id,
        flag: row.flag,
        flag_joined: row.flag_joined,
        prompt: row.prompt,
        default: row.default,
        choice_fixed: row.choice_fixed,
        choice_source: row.choice_source,
        choices: None,
    })
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(rec)
}

async fn get_task_template_arg_choices_by_task_template_arg_id_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<TaskTemplateArgChoices, BackendError> {
    let rec = sqlx::query_as!(TaskTemplateArgChoice, r#"
SELECT
    id,
    task_template_arg_id,
    to_arg,
    label
FROM task_template_arg_choice
WHERE task_template_arg_id = ?1
"#,
        id,
    )
    .fetch_all(&*sqlite.pool)
    .await?;
    Ok(rec.into())
}

#[async_trait]
impl TaskTemplateBackend for SqliteBackend {
    async fn add_new_task_template(
        &self,
        bin_path: &str,
        version_id: &str,
    ) -> Result<i64, BackendError> {
        add_task_template_sqlite(&self, bin_path, version_id).await
    }

    async fn finalize_new_task_template(
        &self,
        id: i64,
    ) -> Result<i64, BackendError> {
        finalize_task_template_sqlite(&self, id).await
    }

    async fn add_task_template_arg(
        &self,
        task_template_id: i64,
        flag: Option<&str>,
        flag_joined: bool,
        prompt: Option<&str>,
        default: Option<&str>,
        choice_fixed: bool,
        choice_source: Option<&str>,
    ) -> Result<i64, BackendError> {
        add_task_template_arg_sqlite(
            &self,
            task_template_id,
            flag,
            flag_joined,
            prompt,
            default,
            choice_fixed,
            choice_source,
        ).await
    }

    async fn delete_task_template_arg_by_id(
        &self,
        id: i64,
    ) -> Result<Option<TaskTemplateArg>, BackendError> {
        delete_task_template_arg_by_id_sqlite(
            &self,
            id,
        ).await
    }

    async fn add_task_template_arg_choice(
        &self,
        task_template_arg_id: i64,
        to_arg: Option<&str>,
        label: &str,
    ) -> Result<i64, BackendError> {
        add_task_template_arg_choice_sqlite(
            &self,
            task_template_arg_id,
            to_arg,
            label,
        ).await
    }

    async fn get_task_template_arg_by_id(
        &self,
        id: i64,
    ) -> Result<Option<TaskTemplateArg>, BackendError>{
        get_task_template_arg_by_id_sqlite(
            &self,
            id,
            true,
        ).await
    }

    async fn delete_task_template_arg_choice_by_id(
        &self,
        id: i64,
    ) -> Result<Option<TaskTemplateArgChoice>, BackendError> {
        delete_task_template_arg_choice_by_id_sqlite(
            &self,
            id,
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
    ) -> Result<i64, BackendError> {
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
    ) -> Result<TaskTemplate, BackendError> {
        let mut result = get_task_template_by_id_sqlite(&self, id).await?;
        let mut args = get_task_template_args_by_task_template_id_sqlite(
            &self, result.id
        ).await?;

        future::try_join_all(args.iter_mut().map(|arg| async {
            Ok::<(), BackendError>(arg.choices = Some(
                get_task_template_arg_choices_by_task_template_arg_id_sqlite(
                    &self,
                    arg.id,
                ).await?
            ))
        })).await?;

        result.args = Some(args);
        Ok(result)
    }

    async fn get_task_template_by_arg_id(
        &self,
        id: i64,
    ) -> Result<TaskTemplate, BackendError> {
        let mut result = get_task_template_by_arg_id_sqlite(&self, id).await?;
        // TODO the following duplicates the above; will need to investigate
        // how to better incorporate these additional selects into the function
        // or provide additional functions/arguments/etc
        let mut args = get_task_template_args_by_task_template_id_sqlite(
            &self, result.id
        ).await?;

        future::try_join_all(args.iter_mut().map(|arg| async {
            Ok::<(), BackendError>(arg.choices = Some(
                get_task_template_arg_choices_by_task_template_arg_id_sqlite(
                    &self,
                    arg.id,
                ).await?
            ))
        })).await?;

        result.args = Some(args);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use pmrmodel_base::task_template::{
        TaskTemplate,
        TaskTemplateArg,
        TaskTemplateArgChoice,
        traits::TaskTemplateBackend,
    };
    use crate::backend::db::{
        Profile,
        SqliteBackend,
    };

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
            args: Some([].to_vec().into()),
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
                default: None,
                choice_fixed: false,
                choice_source: None,
                choices: Some([].to_vec().into()),
            }, TaskTemplateArg {
                id: 2,
                task_template_id: 1,
                flag: None,
                flag_joined: false,
                prompt: Some("Second statement".into()),
                default: None,
                choice_fixed: false,
                choice_source: None,
                choices: Some([].to_vec().into()),
            }].to_vec().into()),
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
                    "default": null,
                    "choice_fixed": false,
                    "choice_source": null,
                    "choices": []
                },
                {
                    "id": 2,
                    "task_template_id": 1,
                    "flag": null,
                    "flag_joined": false,
                    "prompt": "Second statement",
                    "default": null,
                    "choice_fixed": false,
                    "choice_source": null,
                    "choices": []
                }
            ]
        }
        "#).unwrap());

        // add a couple choices
        TaskTemplateBackend::add_task_template_arg_choice(
            &backend, 2, None, "omit").await.unwrap();
        TaskTemplateBackend::add_task_template_arg_choice(
            &backend, 2, Some(""), "empty string").await.unwrap();
        let template = TaskTemplateBackend::get_task_template_by_id(
            &backend, id
        ).await.unwrap();

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
                    "default": null,
                    "choice_fixed": false,
                    "choice_source": null,
                    "choices": []
                },
                {
                    "id": 2,
                    "task_template_id": 1,
                    "flag": null,
                    "flag_joined": false,
                    "prompt": "Second statement",
                    "default": null,
                    "choice_fixed": false,
                    "choice_source": null,
                    "choices": [
                        {
                            "id": 1,
                            "task_template_arg_id": 2,
                            "to_arg": null,
                            "label": "omit"
                        },
                        {
                            "id": 2,
                            "task_template_arg_id": 2,
                            "to_arg": "",
                            "label": "empty string"
                        }
                    ]
                }
            ]
        }
        "#).unwrap());
        assert_eq!(template.args.unwrap()[1].choices, Some([
            TaskTemplateArgChoice {
                id: 1,
                task_template_arg_id: 2,
                to_arg: None,
                label: "omit".into(),
            },
            TaskTemplateArgChoice {
                id: 2,
                task_template_arg_id: 2,
                to_arg: Some("".into()),
                label: "empty string".into(),
            },
        ].to_vec().into()));

        let arg = TaskTemplateBackend::get_task_template_arg_by_id(
            &backend, 2).await.unwrap().unwrap();
        assert_eq!(arg.prompt, Some("Second statement".into()));
        assert_eq!(arg.choices, Some([
            TaskTemplateArgChoice {
                id: 1,
                task_template_arg_id: 2,
                to_arg: None,
                label: "omit".into(),
            },
            TaskTemplateArgChoice {
                id: 2,
                task_template_arg_id: 2,
                to_arg: Some("".into()),
                label: "empty string".into(),
            },
        ].to_vec().into()));

        TaskTemplateBackend::delete_task_template_arg_choice_by_id(
            &backend, 2).await.unwrap();
        let template = TaskTemplateBackend::get_task_template_by_id(
            &backend, id
        ).await.unwrap();
        assert_eq!(template.args.unwrap()[1].choices, Some([
            TaskTemplateArgChoice {
                id: 1,
                task_template_arg_id: 2,
                to_arg: None,
                label: "omit".into(),
            },
        ].to_vec().into()));
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
        TaskTemplateBackend::add_task_template_arg(
            &backend, 1, Some("-i"), false, None, None, false, None
        ).await.unwrap();
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
            args: Some([TaskTemplateArg {
                id: 1,
                task_template_id: 1,
                flag: Some("-i".into()),
                flag_joined: false,
                prompt: None,
                default: None,
                choice_fixed: false,
                choice_source: None,
                choices: Some([].to_vec().into()),
            }].to_vec().into()),
        });

        TaskTemplateBackend::finalize_new_task_template(
            &backend, id,
        ).await.unwrap();

        // deleting should fail
        assert_eq!(None, TaskTemplateBackend::delete_task_template_arg_by_id(
            &backend, 1).await.unwrap());

        let template = TaskTemplateBackend::get_task_template_by_id(
            &backend, id
        ).await.unwrap();
        assert_eq!(template, TaskTemplate {
            id: 1,
            bin_path: "/bin/true".into(),
            version_id: "1.0.0".into(),
            created_ts: 1234567890,
            final_task_template_arg_id: Some(1),
            superceded_by_id: None,
            args: Some([TaskTemplateArg {
                id: 1,
                task_template_id: 1,
                flag: Some("-i".into()),
                flag_joined: false,
                prompt: None,
                default: None,
                choice_fixed: false,
                choice_source: None,
                choices: Some([].to_vec().into()),
            }].to_vec().into()),
        });
    }

    #[async_std::test]
    async fn test_add_rm() {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await
            .unwrap()
            .run_migration_profile(Profile::Pmrtqs)
            .await
            .unwrap();

        let id = TaskTemplateBackend::add_new_task_template(
            &backend, "/bin/true", "1.0.0",
        ).await.unwrap();
        TaskTemplateBackend::add_task_template_arg(
            &backend, 1, Some("-i"), false, None, None, false, None
        ).await.unwrap();
        assert_eq!(TaskTemplateBackend::delete_task_template_arg_by_id(
            &backend, 1).await.unwrap(), Some(TaskTemplateArg {
                id: 1,
                task_template_id: 1,
                flag: Some("-i".into()),
                flag_joined: false,
                prompt: None,
                default: None,
                choice_fixed: false,
                choice_source: None,
                choices: None,
            }
        ));

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
            args: Some([].to_vec().into()),
        });
    }

    #[async_std::test]
    async fn test_add_manual_finalize_nospill() {
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
            args: Some([].to_vec().into()),
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
            args: Some([].to_vec().into()),
        });

        // doing a manual insert to avoid API changes that may prevent this
        sqlx::query!(
            r#"
            INSERT INTO task_template_arg (
                task_template_id,
                flag,
                flag_joined,
                prompt,
                'default',
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
            args: Some([].to_vec().into()),
        });

    }

}