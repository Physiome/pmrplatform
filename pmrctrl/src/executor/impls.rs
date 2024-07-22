use async_trait::async_trait;
use pmrcore::task::TaskDetached;
use pmrtqs::executor::traits;
use tokio::sync::broadcast;

use crate::{
    error::PlatformError,
    executor::Executor,
    handle::TaskExecutorCtrl,
    platform::Platform,
};

impl Executor {
    pub fn new(platform: Platform) -> Self {
        Self { platform }
    }
}

#[async_trait]
impl traits::Executor for Executor {
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
