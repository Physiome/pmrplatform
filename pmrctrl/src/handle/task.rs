use pmrtqs::executor::TMPlatformExecutorInstance;

use crate::platform::Platform;

pub struct TaskExecutorCtrl<'p> {
    pub(crate) platform: &'p Platform,
    pub(crate) executor: TMPlatformExecutorInstance<'p>,
}

mod impls;
