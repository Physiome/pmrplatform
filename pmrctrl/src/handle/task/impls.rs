use pmrcore::{
    task::TaskRef,
};

use crate::{
    error::PlatformError,
    handle::TaskExecutorCtrl,
    platform::Platform,
};

impl<'p> TaskExecutorCtrl<'p> {
    pub(crate) fn new(
        platform: &'p Platform,
        task: TaskRef<'p>,
    ) -> Self {
        Self {
            platform,
            executor: task.into(),
        }
    }

    pub async fn execute(mut self) -> Result<(i32, bool), PlatformError> {
        let (exit_status, _) = self.executor.execute().await?;
        Ok((exit_status, self.platform.complete_task(
            self.executor.into(),
            exit_status.into(),
        ).await?))
    }
}
