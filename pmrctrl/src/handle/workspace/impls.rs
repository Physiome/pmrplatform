use pmrcore::workspace::{
    traits::Workspace as _,
    WorkspaceRef,
};
use pmrrepo::handle::GitHandle;
use std::fmt;

use crate::{
    error::PlatformError,
    platform::Platform,
};

use super::WorkspaceCtrl;

impl fmt::Debug for WorkspaceCtrl<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkspaceCtrl<'_>")
            .field("platform", &self.platform)
            .field("workspace.id", &self.workspace.id())
            .finish()
    }
}

impl<'p> WorkspaceCtrl<'p> {
    pub(crate) fn new(
        platform: &'p Platform,
        workspace: WorkspaceRef<'p>
    ) -> Self {
        Self {
            platform,
            workspace,
            handle: Default::default(),
        }
    }

    pub fn workspace(&self) -> &WorkspaceRef<'p> {
        &self.workspace
    }

    pub async fn alias(&self) -> Result<Option<String>, PlatformError> {
        Ok(self.platform.mc_platform.get_alias(
            "workspace",
            self.workspace.id(),
        ).await?)
    }
}
