use pmrcore::workspace::WorkspaceRef;

use crate::platform::Platform;

pub struct WorkspaceCtrl<'p> {
    pub(crate) platform: &'p Platform,
    pub(crate) workspace: WorkspaceRef<'p>,
}

mod impls;
