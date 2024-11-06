use pmrcore::workspace::WorkspaceRef;
use pmrrepo::handle::GitHandle;
use std::sync::OnceLock;

use crate::platform::Platform;

pub struct WorkspaceCtrl<'p> {
    pub(crate) platform: &'p Platform,
    pub(crate) workspace: WorkspaceRef<'p>,
    pub(crate) handle: OnceLock<GitHandle<'p>>,
}

mod impls;
