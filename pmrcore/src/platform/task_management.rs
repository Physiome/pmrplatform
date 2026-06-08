use async_trait::async_trait;
use crate::{
    error::BackendError,
    platform::{
        PlatformCore,
        RawPlatform,
    },
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
    + PlatformCore

    + Send
    + Sync
{
    fn as_dyn(&self) -> &dyn TMPlatform;

    async fn start_task(
        &self,
    ) -> Result<Option<TaskRef>, BackendError> {
        Ok(TaskBackend::start(self)
            .await?
            .map(|task| task.bind(self.as_dyn()))
        )
    }
}

pub trait DefaultTMPlatform: TMPlatform {}
pub trait RawTMPlatform: TMPlatform + RawPlatform {}

impl<P: TaskBackend
    + TaskTemplateBackend
    + PlatformCore

    + DefaultTMPlatform

    + Send
    + Sync
> TMPlatform for P {
    fn as_dyn(&self) -> &dyn TMPlatform {
        self
    }
}

impl <P: TMPlatform + RawPlatform> RawTMPlatform for P {}
