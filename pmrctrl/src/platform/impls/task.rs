use pmrcore::{
    task::{
        Task,
        TaskRef,
        traits::TaskBackend,
    },
    exposure::task::traits::ExposureTaskBackend,
};
use crate::{
    error::PlatformError,
    handle::TaskExecutorCtrl,
    platform::Platform,
};

impl<'p> Platform {
    pub async fn adds_task(
        &self,
        task: Task,
    ) -> Result<Task, PlatformError> {
        Ok(TaskBackend::adds_task(self.tm_platform(), task).await?)
    }

    pub async fn start_task(
        &'p self,
    ) -> Result<Option<TaskExecutorCtrl<'p>>, PlatformError> {
        Ok(self.tm_platform()
            .start_task()
            .await?
            .map(|t| TaskExecutorCtrl::new(&self, t))
        )
    }

    pub async fn complete_task(
        &self,
        mut task: TaskRef<'_>,
        exit_status: i64,
    ) -> Result<bool, PlatformError> {
        let task_id = task.id();
        task.complete(exit_status).await
            .map_err(|e| {
                log::error!(
                    "Task:{task_id} failed to write to db the value of exit_status: {}; error: {}",
                    exit_status,
                    e,
                );
                e
            })?;
        // TODO figure out if we need to record task run failure for the
        // exposure task log
        if exit_status == 0 {
            Ok(match ExposureTaskBackend::finalize_task_id(
                self.mc_platform(),
                task_id,
            ).await {
                Ok(Some((id, Some(view_key)))) => {
                    log::debug!("Task:{task_id} ran for ExposureFileView:{id}, produced view {view_key}");
                    true
                }
                Ok(Some((id, None))) => {
                    log::warn!("Task:{task_id} ran for ExposureFileView:{id}, but failed to produced view");
                    false
                }
                Ok(None) => {
                    // TODO we've somehow triggered this with sqlite.
                    log::warn!("Task:{task_id} ran but it failed to produce results?");
                    false
                }
                Err(err) => {
                    log::error!("Task:{task_id} failed to be finalized into the db; error: {err}");
                    return Err(err)?;
                }
            })
        } else {
            log::debug!("Task:{task_id} ran but its task exited with {exit_status}");
            Ok(false)
        }
    }
}
