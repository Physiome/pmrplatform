use async_trait::async_trait;
use crate::{
    error::BackendError,
    task::{
        TaskRef,
        traits::TaskBackend,
    },
    task_template::traits::TaskTemplateBackend,
};

/// TMPlatform - Task Management Platform
///
/// This platform is used to manage the tasks for PMR, from setting up
/// the task templates to queuing of tasks.
///
/// This trait is applicable to everything that correctly implements the
/// relevant backends that compose this trait.
#[async_trait]
pub trait TMPlatform: TaskBackend
    + TaskTemplateBackend
{
    async fn start_task(
        &self,
    ) -> Result<Option<TaskRef<Self>>, BackendError>
    where Self: Sized
    {
        Ok(TaskBackend::start(self)
            .await?
            .map(|task| task.bind(self))
        )
    }
}

impl<P: TaskBackend
    + TaskTemplateBackend
> TMPlatform for P {}
