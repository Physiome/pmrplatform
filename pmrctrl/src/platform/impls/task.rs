use pmrcore::{
    task::{
        Task,
        traits::TaskBackend,
    },
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
}
