use crate::platform::Platform;

#[derive(Clone)]
pub struct Executor {
    pub(crate) platform: Platform,
}
