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
    platform::Platform,
};

impl<
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
> Platform<MCP, TMP> {
    pub async fn adds_task(
        &self,
        task: Task,
    ) -> Result<Task, PlatformError> {
        Ok(TaskBackend::adds_task(self.tm_platform.as_ref(), task).await?)
    }

    // TODO determine how to call tm_platform.start_task()

    pub async fn complete_task(
        &self,
        mut task: TaskRef<'_, TMP>,
        exit_status: i64,
    ) -> Result<bool, PlatformError> {
        task.complete(exit_status).await?;
        // TODO figure out if we need to record task run failure for the
        // exposure task log
        if exit_status == 0 {
            let etb: &dyn ExposureTaskBackend = self.mc_platform.as_ref();
            Ok(etb.finalize_task_id(task.id()).await?)
        } else {
            Ok(false)
        }
    }
}
