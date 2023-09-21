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
    'a,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> Platform<'a, MCP, TMP> {
    pub async fn adds_task(
        &self,
        task: Task,
    ) -> Result<Task, PlatformError> {
        Ok(TaskBackend::adds_task(&self.tm_platform, task).await?)
    }
}
