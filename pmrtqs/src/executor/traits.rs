use async_trait::async_trait;
use pmrcore::task::TaskDetached;
use tokio::sync::broadcast;

#[async_trait]
pub trait Executor {
    type Error;

    async fn start_task(
        &self,
    ) -> Result<Option<TaskDetached>, Self::Error>;
    async fn execute(
        &self,
        task: TaskDetached,
        abort_receiver: broadcast::Receiver<()>,
    ) -> Result<(i32, bool), Self::Error>;
}
