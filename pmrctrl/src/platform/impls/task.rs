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
        Ok(TaskBackend::adds_task(self.tm_platform.as_ref(), task).await?)
    }

    pub async fn start_task(
        &'p self,
    ) -> Result<Option<TaskExecutorCtrl<'p>>, PlatformError> {
        Ok(self.tm_platform.as_ref()
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
        task.complete(exit_status).await?;
        // TODO figure out if we need to record task run failure for the
        // exposure task log
        if exit_status == 0 {
            let task_id = task.id();
            Ok(match ExposureTaskBackend::finalize_task_id(
                self.mc_platform.as_ref(),
                task_id,
            ).await? {
                Some((id, Some(view_key))) => {
                    log::debug!("Task:{task_id} ran for ExposureFileView:{id}, produced view {view_key}");
                    true
                }
                Some((id, None)) => {
                    log::warn!("Task:{task_id} ran for ExposureFileView:{id}, but failed to produced view");
                    false
                }
                None => {
                    // TODO we've somehow triggered this with sqlite.
                    log::warn!("Task:{task_id} ran but it failed to produce results?");
                    false
                }
            })
        } else {
            Ok(false)
        }
    }
}
