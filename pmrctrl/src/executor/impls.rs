use async_trait::async_trait;
use pmrcore::{
    platform::{
        MCPlatform,
        TMPlatform,
    },
    task::TaskDetached,
};
use pmrtqs::executor::traits;
use tokio::sync::broadcast;

use crate::{
    error::PlatformError,
    executor::Executor,
    handle::TaskExecutorCtrl,
    platform::Platform,
};

impl<
    MCP: MCPlatform + Clone + Send + Sync,
    TMP: TMPlatform + Clone + Send + Sync,
> Executor<MCP, TMP> {
    pub fn new(platform: Platform<MCP, TMP>) -> Self {
        Self { platform }
    }
}

#[async_trait]
impl<
    MCP: MCPlatform + Clone + Send + Sync,
    TMP: TMPlatform + Clone + Send + Sync,
> traits::Executor for Executor<MCP, TMP> {
    type Error = PlatformError;

    async fn start_task(
        &self,
    ) -> Result<Option<TaskDetached>, Self::Error> {
        Ok(self.platform
            .tm_platform
            .as_ref()
            .start_task()
            .await
            .map(|task| task.map(|task| task.detach()))?
        )
    }

    async fn execute(
        &self,
        task: TaskDetached,
        // TODO deal with aborts.
        _abort_receiver: broadcast::Receiver<()>,
    ) -> Result<(i32, bool), Self::Error> {
        let tec = TaskExecutorCtrl::new(
            &self.platform,
            task.bind(self.platform.tm_platform.as_ref())?,
        );
        tec.execute().await
    }
}
