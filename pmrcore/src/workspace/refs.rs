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

pub struct WorkspaceRef<'a> {
    pub(super) inner: Workspace,
    pub(super) exposures: OnceLock<exposure::ExposureRefs<'a>>,
    pub(super) platform: &'a dyn MCPlatform,
}

pub struct WorkspaceSyncRef<'a> {
    pub(super) id: i64,
    pub(super) platform: &'a dyn MCPlatform,
}

pub struct WorkspaceRefs<'a>(pub(super) Vec<WorkspaceRef<'a>>);

impl Workspace {
    pub(crate) fn bind<'a>(
        self,
        platform: &'a dyn MCPlatform,
    ) -> WorkspaceRef<'a> {
        WorkspaceRef {
            inner: self,
            exposures: OnceLock::new(),
            platform: platform,
        }
    }
}

impl Workspaces {
    pub(crate) fn bind<'a>(
        self,
        platform: &'a dyn MCPlatform,
    ) -> WorkspaceRefs<'a> {
        self.0
            .into_iter()
            .map(|v| v.bind(platform))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<'a> WorkspaceRef<'a> {
    pub fn into_inner(self) -> Workspace {
        self.inner
    }

    pub fn clone_inner(&self) -> Workspace {
        self.inner.clone()
    }

    pub async fn begin_sync(&self) -> Result<WorkspaceSyncRef<'a>, BackendError> {
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

impl WorkspaceSyncRef<'_> {
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
