use crate::{
    error::BackendError,
    platform::TMPlatform,
    task::{
        Task,
        traits::{
            TaskBackend,
        },
    },
};

pub struct TaskRef<'a> {
    pub(super) inner: Task,
    pub(super) platform: &'a dyn TMPlatform,
}

impl Task {
    pub(crate) fn bind<'a>(
        self,
        platform: &'a dyn TMPlatform,
    ) -> TaskRef<'a> {
        TaskRef {
            inner: self,
            platform: platform,
        }
    }
}

impl TaskRef<'_> {
    pub async fn run(
        &mut self,
        pid: i64,
    ) -> Result<bool, BackendError> {
        // self.platform.run(self.inner.id, pid).await
        let result = TaskBackend::run(
            self.platform,
            self.inner.id,
            pid,
        ).await?;
        self.inner.pid = Some(pid);
        Ok(result)
    }

    pub async fn complete(
        &mut self,
        exit_status: i64,
    ) -> Result<bool, BackendError> {
        // self.platform.complete(self.inner.id, exit_status).await
        let result = TaskBackend::complete(
            self.platform,
            self.inner.id,
            exit_status,
        ).await?;
        self.inner.exit_status = Some(exit_status);
        Ok(result)
    }

    // TODO maybe move this to a common trait like what was done with
    // Exposure types.

    pub fn id(&self) -> i64 {
        self.inner.id
    }

    pub fn pid(&self) -> Option<i64> {
        self.inner.pid
    }

    pub fn exit_status(&self) -> Option<i64> {
        self.inner.exit_status
    }
}

impl TaskRef<'_> {
    pub fn into_inner(self) -> Task {
        self.inner
    }
}
