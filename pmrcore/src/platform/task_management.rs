use crate::{
    task::traits::TaskBackend,
    task_template::traits::TaskTemplateBackend,
};

/// TMPlatform - Task Management Platform
///
/// This platform is used to manage the tasks for PMR, from setting up
/// the task templates to queuing of tasks.
///
/// This trait is applicable to everything that correctly implements the
/// relevant backends that compose this trait.
pub trait TMPlatform: TaskBackend
    + TaskTemplateBackend
{
}

impl<P: TaskBackend
    + TaskTemplateBackend
> TMPlatform for P {}
