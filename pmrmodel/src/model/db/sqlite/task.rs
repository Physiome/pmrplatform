use async_trait::async_trait;
#[cfg(not(test))]
use chrono::Utc;
use pmrcore::{
    error::{
        BackendError,
        task::TaskError,
    },
    task::{
        Task,
        TaskArg,
        traits::TaskBackend,
    },
};

use crate::{
    backend::db::SqliteBackend,
};

#[cfg(test)]
use crate::test::Utc;

// TODO document that adds means _add_ing (insert) the _s_tructure.

async fn adds_task_sqlite(
    sqlite: &SqliteBackend,
    task: Task,
) -> Result<Task, TaskError> {
    if task.id > 0 {
        return Err(TaskError::TaskAlreadyQueued(task.id));
    }

    let mut tx = sqlite.pool.begin().await
        .map_err(BackendError::from)?;
    let created_ts = Utc::now().timestamp();
    let mut result = Task {
        bin_path: task.bin_path.clone(),
        created_ts: created_ts,
        basedir: task.basedir.clone(),
        .. Default::default()
    };

    result.id = sqlx::query!(
        "
INSERT INTO task (
    task_template_id,
    bin_path,
    created_ts,
    basedir
)
VALUES ( ?1, ?2, ?3, ?4 )\
        ",
        task.task_template_id,
        task.bin_path,
        created_ts,
        task.basedir,
    ).execute(&mut *tx)
        .await.map_err(BackendError::from)?
        .last_insert_rowid();

    result.args = match task.args {
        Some(args) => {
            let mut iter = args.iter().into_iter();
            let mut results: Vec<TaskArg> = Vec::new();
            while let Some(arg) = iter.next() {
                // TaskArg on the other hand won't have a direct API for
                // insertion, so if any of them containing an id will
                // simply have a warning generated instead.
                if arg.id > 0 {
                    log::warn!("got an existing id for arg: {}", arg.id);
                }
                if arg.task_id > 0 {
                    log::warn!("got an existing task_id for arg: {}", arg.task_id);
                }
                let task_arg_id = sqlx::query!(
                        "
INSERT INTO task_arg (
    task_id,
    arg
)
VALUES ( ?1, ?2 )\
                        ",
                        result.id,
                        arg.arg,
                    ).execute(&mut *tx)
                    .await.map_err(BackendError::from)?
                    .last_insert_rowid();
                results.push(TaskArg {
                    id: task_arg_id,
                    task_id: result.id,
                    arg: arg.arg.clone(),
                })
            }
            Some(results.into())
        }
        None => None,
    };
    tx.commit().await.map_err(BackendError::from)?;
    Ok(result)
}

#[async_trait]
impl TaskBackend for SqliteBackend {
    async fn adds_task(
        &self,
        task: Task,
    ) -> Result<Task, TaskError> {
        adds_task_sqlite(&self, task).await
    }
}

#[cfg(test)]
mod tests {
    use pmrcore::task_template::traits::TaskTemplateBackend;
    use pmrcore::task::{
        Task,
        TaskArg,
        traits::TaskBackend,
    };
    use crate::backend::db::{
        Profile,
        SqliteBackend,
    };

    #[async_std::test]
    async fn test_adds_task() {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await
            .unwrap()
            .run_migration_profile(Profile::Pmrtqs)
            .await
            .unwrap();

        let (id, _) = TaskTemplateBackend::add_task_template(
            &backend, "/bin/true", "1.0.0",
        ).await
            .unwrap();
        TaskTemplateBackend::finalize_new_task_template(
            &backend, id,
        ).await.unwrap();

        // note that no arguments were added to the task template, but
        // arguments are injected here - the model API should be used to
        // create the following normally so validation should have done
        // there.
        let task = Task {
            task_template_id: id,
            bin_path: "/bin/demo".into(),
            basedir: "/tmp".into(),
            args: Some(["--format=test", "-t", "standard" ].iter()
                .map(|a| TaskArg {
                    arg: a.to_string(),
                    .. Default::default()
                })
                    .collect::<Vec<_>>()
                    .into()),
            .. Default::default()
        };

        let task = TaskBackend::adds_task(
            &backend, task,
        ).await
            .unwrap();

        // additional assigned ids/values are all in place.
        assert_eq!(task.id, 1);
        assert_eq!(task.created_ts, 1234567890);
        assert_eq!(&task.bin_path, "/bin/demo");
        assert_eq!(&task.basedir, "/tmp");
        assert_eq!(task.args, Some(serde_json::from_str(r#"[
            {
                "id": 1,
                "task_id": 1,
                "arg": "--format=test"
            },
            {
                "id": 2,
                "task_id": 1,
                "arg": "-t"
            },
            {
                "id": 3,
                "task_id": 1,
                "arg": "standard"
            }
        ]"#).unwrap()));

    }
}
