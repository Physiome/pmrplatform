use std::fmt;
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

impl fmt::Display for TaskDetached {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TaskDetached {{ id: {} }}", self.inner.id)
    }
}

impl TaskRef<'_> {
    pub fn detach(self) -> TaskDetached {
        TaskDetached {
            inner: self.inner,
            url: self.platform.url().to_string(),
        }
    }
}

impl TaskDetached {
    pub fn bind<'a>(
        self,
        platform: &'a dyn TMPlatform,
    ) -> Result<TaskRef<'a>, BackendError> {
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
