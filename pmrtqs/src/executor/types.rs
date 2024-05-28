use pmrcore::{
    platform::TMPlatform,
    task::TaskRef,
};

#[derive(Clone)]
pub struct TMPlatformExecutor<P: Clone> {
    pub(crate) platform: P,
}

pub struct TMPlatformExecutorInstance<'a, P: TMPlatform + Sync> {
    pub(crate) task: TaskRef<'a, P>,
}
