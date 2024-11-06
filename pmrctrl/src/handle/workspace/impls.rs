use pmrcore::workspace::{
    traits::Workspace as _,
    WorkspaceRef,
};
use pmrrepo::handle::GitHandle;
use std::fmt;

use crate::platform::Platform;

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
}
