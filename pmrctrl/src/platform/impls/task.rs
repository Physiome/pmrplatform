use pmrcore::{
    task::{
        Task,
        TaskRef,
        traits::TaskBackend,
    },
    exposure::task::traits::ExposureTaskBackend,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use crate::{
    error::PlatformError,
    handle::TaskExecutorCtrl,
    platform::Platform,
};

impl<
    'p,
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
> Platform<MCP, TMP> {
    pub async fn adds_task(
        &self,
        task: Task,
    ) -> Result<Task, PlatformError> {
        Ok(TaskBackend::adds_task(self.tm_platform.as_ref(), task).await?)
    }

    pub async fn start_task(
        &'p self,
    ) -> Result<Option<TaskExecutorCtrl<'p, MCP, TMP>>, PlatformError> {
        Ok(self.tm_platform.as_ref()
            .start_task()
            .await?
            .map(|t| TaskExecutorCtrl::new(&self, t))
        )
    }

    pub async fn complete_task(
        &self,
        mut task: TaskRef<'_, TMP>,
        exit_status: i64,
    ) -> Result<bool, PlatformError> {
        task.complete(exit_status).await?;
        // TODO figure out if we need to record task run failure for the
        // exposure task log
        if exit_status == 0 {
            Ok(ExposureTaskBackend::finalize_task_id(
                self.mc_platform.as_ref(),
                task.id(),
            ).await?)
        } else {
            Ok(false)
        }
    }
}
