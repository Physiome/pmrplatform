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
    /// Start a task. This should pick the oldest task that was added,
    /// and atomically update its start_ts and return the complete task
    /// instance.
    ///
    /// Returns some complete task instance, or none if no such task is
    /// found.
    async fn start(
        &self,
    ) -> Result<Option<Task>, BackendError>;
}

