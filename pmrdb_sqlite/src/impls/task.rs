use async_trait::async_trait;
use pmrcore::{
    error::{
        BackendError,
        task::TaskError,
    },
    task::{
        Task,
        TaskArg,
        TaskArgs,
        traits::TaskBackend,
    },
};

use crate::{
    SqliteBackend,
    chrono::Utc,
};

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

async fn gets_task_args_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<TaskArgs, BackendError> {
    Ok(sqlx::query_as!(
        TaskArg,
        "
SELECT
    id,
    task_id,
    arg
FROM
    task_arg
WHERE
    task_id = ?1
ORDER BY
    id
        ",
        id,
    )
        .fetch_all(&*sqlite.pool)
        .await?
        .into()
    )
}

async fn gets_task_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
) -> Result<Task, BackendError> {
    let mut result = sqlx::query!(
        "
SELECT
    id,
    task_template_id,
    bin_path,
    pid,
    created_ts,
    start_ts,
    stop_ts,
    exit_status,
    basedir
FROM
    task
WHERE
    id = ?1
        ",
        id,
    )
        .map(|row| Task {
            id: row.id,
            task_template_id: row.task_template_id,
            bin_path: row.bin_path,
            pid: row.pid,
            created_ts: row.created_ts,
            start_ts: row.start_ts,
            stop_ts: row.stop_ts,
            exit_status: row.exit_status,
            basedir: row.basedir,
            args: None,
        })
        .fetch_one(&*sqlite.pool)
        .await?;
    result.args = Some(gets_task_args_sqlite(sqlite, id).await?);

    Ok(result)
}

async fn start_task_sqlite(
    sqlite: &SqliteBackend,
) -> Result<Option<Task>, BackendError> {
    let start_ts = Utc::now().timestamp();
    // the query assumes the id is auto-incremented to have the earliest
    // task has the lowest id.
    let mut result = sqlx::query!(
        "
UPDATE
    task
SET
    start_ts = ?1
WHERE id = (
    SELECT
        id
    FROM
        task
    WHERE
        start_ts IS NULL
    ORDER BY
        id
    LIMIT 1
)
RETURNING
    id,
    task_template_id,
    bin_path,
    pid,
    created_ts,
    start_ts,
    stop_ts,
    exit_status,
    basedir
        ",
        start_ts,
    )
        .map(|row| Task {
            id: row.id,
            task_template_id: row.task_template_id,
            bin_path: row.bin_path,
            pid: row.pid,
            created_ts: row.created_ts,
            start_ts: row.start_ts,
            stop_ts: row.stop_ts,
            exit_status: row.exit_status,
            basedir: row.basedir,
            args: None,
        })
        .fetch_optional(&*sqlite.pool)
        .await?;
    match result.as_mut() {
        Some(result) => result.args = Some(
            gets_task_args_sqlite(sqlite, result.id).await?
        ),
        None => (),
    }
    Ok(result)
}

async fn run_task_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
    pid: i64,
) -> Result<bool, BackendError> {
    let rows_affected = sqlx::query!(
        "
UPDATE
    task
SET
    pid = ?2
WHERE id = ?1
        ",
        id,
        pid,
    )
        .execute(&*sqlite.pool)
        .await?
        .rows_affected();
    Ok(rows_affected > 0)
}

async fn complete_task_sqlite(
    sqlite: &SqliteBackend,
    id: i64,
    exit_status: i64,
) -> Result<bool, BackendError> {
    let stop_ts = Utc::now().timestamp();
    let rows_affected = sqlx::query!(
        "
UPDATE
    task
SET
    stop_ts = ?2,
    exit_status = ?3
WHERE id = ?1
        ",
        id,
        stop_ts,
        exit_status,
    )
        .execute(&*sqlite.pool)
        .await?
        .rows_affected();
    Ok(rows_affected > 0)
}


#[async_trait]
impl TaskBackend for SqliteBackend {
    async fn adds_task(
        &self,
        task: Task,
    ) -> Result<Task, TaskError> {
        adds_task_sqlite(&self, task).await
    }
    async fn gets_task(
        &self,
        id: i64,
    ) -> Result<Task, BackendError> {
        gets_task_sqlite(&self, id).await
    }
    async fn start(
        &self,
    ) -> Result<Option<Task>, BackendError> {
        start_task_sqlite(&self).await
    }
    async fn run(
        &self,
        id: i64,
        pid: i64,
    ) -> Result<bool, BackendError> {
        run_task_sqlite(&self, id, pid).await
    }
    async fn complete(
        &self,
        id: i64,
        exit_status: i64,
    ) -> Result<bool, BackendError> {
        complete_task_sqlite(&self, id, exit_status).await
    }
}

#[cfg(test)]
mod tests {
    use pmrcore::{
        platform::PlatformBuilder,
        task_template::traits::TaskTemplateBackend,
        task::{
            Task,
            TaskArg,
            traits::TaskBackend,
        },
    };
    use crate::SqliteBackend;

    #[async_std::test]
    async fn test_adds_task() -> anyhow::Result<()> {
        let backend = SqliteBackend::tm("sqlite::memory:")
            .await
            .map_err(anyhow::Error::from_boxed)?;

        // Need the task template to provide a valid reference for the
        // Task that follow this.
        let (id, _) = TaskTemplateBackend::add_task_template(
            &backend, "/bin/true", "1.0.0",
        ).await?;
        TaskTemplateBackend::finalize_new_task_template(
            &backend, id,
        ).await?;

        // Create a Task manually, bypassing the standard helpers that
        // would have required setting up the task template which isn't
        // the focus of the test.
        //
        // Normal usage should go through the relevant APIs for normal
        // error checking and validation.
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
        ).await?;

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
        ]"#)?));

        Ok(())
    }

    #[async_std::test]
    async fn test_start_task() -> anyhow::Result<()> {
        let backend = SqliteBackend::tm("sqlite::memory:")
            .await
            .map_err(anyhow::Error::from_boxed)?;
        let (id, _) = TaskTemplateBackend::add_task_template(
            &backend, "/bin/true", "1.0.0",
        ).await?;
        TaskTemplateBackend::finalize_new_task_template(
            &backend, id,
        ).await?;

        // No tasks added.
        assert!(TaskBackend::start(&backend).await?.is_none());

        let task = TaskBackend::adds_task(
            &backend,
            Task {
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
            }
        ).await?;

        // the only outstanding task should be returned.
        let started_task = TaskBackend::start(&backend)
            .await?
            .expect("a task has started");
        assert_eq!(started_task.id, task.id);
        assert_eq!(started_task.start_ts, Some(1234567890));

        // No outstanding task, so no task can be started.
        assert!(TaskBackend::start(&backend).await?.is_none());

        let task2 = TaskBackend::adds_task(
            &backend,
            Task {
                task_template_id: id,
                bin_path: "/bin/demo".into(),
                basedir: "/tmp".into(),
                .. Default::default()
            }
        ).await?;

        let task3 = TaskBackend::adds_task(
            &backend,
            Task {
                task_template_id: id,
                bin_path: "/bin/demo".into(),
                basedir: "/tmp".into(),
                .. Default::default()
            }
        ).await?;

        let start2 = TaskBackend::start(&backend)
            .await?
            .expect("second task has started");
        assert_eq!(start2.id, task2.id);

        let start3 = TaskBackend::start(&backend)
            .await?
            .expect("third task has started");
        assert_eq!(start3.id, task3.id);
        assert_eq!(start3.start_ts, Some(1234567890));

        Ok(())
    }

    #[async_std::test]
    async fn test_task_complete_flow() -> anyhow::Result<()> {
        let backend = SqliteBackend::tm("sqlite::memory:")
            .await
            .map_err(anyhow::Error::from_boxed)?;
        let (id, _) = TaskTemplateBackend::add_task_template(
            &backend, "/bin/true", "1.0.0",
        ).await?;
        TaskTemplateBackend::finalize_new_task_template(
            &backend, id,
        ).await?;
        let task = TaskBackend::adds_task(
            &backend,
            Task {
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
            }
        ).await?;

        TaskBackend::start(&backend)
            .await?
            .expect("a task has started");

        assert!(TaskBackend::run(&backend, task.id, 123)
            .await?
        );
        let running_task = TaskBackend::gets_task(
            &backend, task.id
        ).await?;
        assert_eq!(running_task.pid, Some(123));

        assert!(TaskBackend::complete(&backend, task.id, 0)
            .await?
        );
        let completed_task = TaskBackend::gets_task(
            &backend, task.id
        ).await?;
        assert_eq!(completed_task.pid, Some(123));
        assert_eq!(completed_task.exit_status, Some(0));

        Ok(())
    }
}
