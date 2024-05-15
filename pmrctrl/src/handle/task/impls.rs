use pmrcore::{
    error::task::TaskError,
    exposure::traits::ExposureFileViewBackend,
    platform::{
        MCPlatform,
        TMPlatform,
    },
    task::TaskRef,
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
> TaskExecutorCtrl<'p, MCP, TMP> {
    pub(crate) fn new(
        platform: &'p Platform<MCP, TMP>,
        task: TaskRef<'p, TMP>,
    ) -> Self {
        Self {
            platform,
            executor: task.into(),
        }
    }

    pub async fn execute(mut self) -> Result<bool, PlatformError> {
        let exit_status = self.executor.execute().await?;
        Ok(self.platform.complete_task(
            self.executor.into(),
            exit_status.into(),
        ).await?)
    }
}
