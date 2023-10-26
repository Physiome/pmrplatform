use async_trait::async_trait;
use crate::{
    error::{
        BackendError,
        task::TaskError,
    },
    task::Task,
};

#[async_trait]
pub trait TaskBackend {
    /// Add a complete task instance.
    async fn adds_task(
        &self,
        task: Task,
    ) -> Result<Task, TaskError>;
    /// Get a complete task instance by its id
    async fn gets_task(
        &self,
        id: i64,
    ) -> Result<Task, BackendError>;
}

