use crate::{
    error::BackendError,
    platform::TMPlatform,
    task::{
        Task,
        TaskRef,
    },
};

pub struct TaskDetached {
    pub(super) inner: Task,
    pub(super) url: String,
}

impl<P: TMPlatform + Sized> TaskRef<'_, P> {
    pub fn detach(self) -> TaskDetached {
        TaskDetached {
            inner: self.inner,
            url: self.platform.url().to_string(),
        }
    }
}

impl TaskDetached {
    pub fn bind<'a, P: TMPlatform + Sized>(
        self,
        platform: &'a P,
    ) -> Result<TaskRef<'a, P>, BackendError> {
        if self.url == platform.url() {
            Ok(TaskRef {
                inner: self.inner,
                platform: platform,
            })
        } else {
            Err(BackendError::NonMatchingBind)
        }
    }

    // TODO maybe move this to a common trait like what was done with
    // Exposure types.

    pub fn id(&self) -> i64 {
        self.inner.id
    }
}
