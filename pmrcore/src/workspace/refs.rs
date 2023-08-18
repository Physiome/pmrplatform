use std::sync::OnceLock;
use crate::{
    error::BackendError,
    exposure,
    platform::MCPlatform,
    workspace::{
        Workspace,
        Workspaces,
        WorkspaceSyncStatus,
        traits::{
            WorkspaceSyncBackend,
        },
    },
};

pub struct WorkspaceRef<'a, P: MCPlatform + Sized> {
    pub(super) inner: Workspace,
    pub(super) exposures: OnceLock<exposure::ExposureRefs<'a, P>>,
    pub(super) platform: &'a P,
}

pub struct WorkspaceSyncRef<'a, P: MCPlatform + Sized> {
    pub(super) id: i64,
    pub(super) platform: &'a P,
}

pub struct WorkspaceRefs<'a, P: MCPlatform + Sized>(pub(super) Vec<WorkspaceRef<'a, P>>);

impl Workspace {
    pub(crate) fn bind<'a, P: MCPlatform + Sized>(
        self,
        platform: &'a P,
    ) -> WorkspaceRef<'a, P> {
        WorkspaceRef {
            inner: self,
            exposures: OnceLock::new(),
            platform: platform,
        }
    }
}

impl Workspaces {
    pub(crate) fn bind<'a, P: MCPlatform + Sized>(
        self,
        platform: &'a P,
    ) -> WorkspaceRefs<'a, P> {
        self.0
            .into_iter()
            .map(|v| v.bind(platform))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<'a, P: MCPlatform + Sized> WorkspaceRef<'a, P> {
    pub fn into_inner(self) -> Workspace {
        self.inner
    }

    pub fn clone_inner(&self) -> Workspace {
        self.inner.clone()
    }

    pub async fn begin_sync(&self) -> Result<WorkspaceSyncRef<'a, P>, BackendError> {
        let id = WorkspaceSyncBackend::begin_sync(
            self.platform,
            self.inner.id,
        ).await?;
        Ok(WorkspaceSyncRef {
            id: id,
            platform: self.platform
        })
    }
}

impl<P: MCPlatform + Sized> WorkspaceSyncRef<'_, P> {
    pub async fn complete_sync(&self) -> Result<bool, BackendError> {
        WorkspaceSyncBackend::complete_sync(
            self.platform,
            self.id,
            WorkspaceSyncStatus::Completed,
        ).await
    }

    pub async fn fail_sync(&self) -> Result<bool, BackendError> {
        WorkspaceSyncBackend::complete_sync(
            self.platform,
            self.id,
            WorkspaceSyncStatus::Error,
        ).await
    }
}
