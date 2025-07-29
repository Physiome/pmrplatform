use pmrcore::{
    platform::TMPlatform,
    task::TaskRef,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct TMPlatformExecutor {
    pub(crate) platform: Arc<dyn TMPlatform>,
}

pub struct TMPlatformExecutorInstance<'a> {
    pub(crate) task: TaskRef<'a>,
}
