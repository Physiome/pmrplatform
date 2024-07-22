use pmrcore::task::TaskRef;

#[derive(Clone)]
pub struct TMPlatformExecutor<P: Clone + Send + Sync> {
    pub(crate) platform: P,
}

pub struct TMPlatformExecutorInstance<'a> {
    pub(crate) task: TaskRef<'a>,
}
