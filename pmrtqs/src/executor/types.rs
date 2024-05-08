use pmrcore::{
    platform::TMPlatform,
    task::TaskRef,
};

pub struct Executor<'a, P: TMPlatform + Sync> {
    pub(crate) task: TaskRef<'a, P>,
}
