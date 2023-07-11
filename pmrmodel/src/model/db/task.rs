use async_trait::async_trait;
use pmrmodel_base::task::{
    Task,
};
use crate::error::TaskError;

#[async_trait]
pub trait TaskBackend {
    // add a new task template that's open to updates
    async fn adds_task(
        &self,
        task: Task,
    ) -> Result<Task, TaskError>;
}

