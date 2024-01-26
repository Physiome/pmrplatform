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

pub struct TaskRef<'a, P: TMPlatform + Sized> {
    pub(super) inner: Task,
    pub(super) platform: &'a P,
}

impl<P: TMPlatform + Sized> TaskRef<'_, P> {
    pub async fn run(
        &self,
        pid: i64,
    ) -> Result<bool, BackendError> {
        // self.platform.run(self.inner.id, pid).await
        TaskBackend::run(
            self.platform,
            self.inner.id,
            pid,
        ).await
    }

    pub async fn complete(
        &self,
        exit_status: i64,
    ) -> Result<bool, BackendError> {
        // self.platform.complete(self.inner.id, exit_status).await
        TaskBackend::complete(
            self.platform,
            self.inner.id,
            exit_status,
        ).await
    }
}
