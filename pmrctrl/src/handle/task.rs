use pmrcore::{
    platform::{
        MCPlatform,
        TMPlatform,
    },
    task::TaskRef,
};

use crate::platform::Platform;

pub struct TaskCtrl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    pub(crate) platform: &'p Platform<'db, MCP, TMP>,
    pub(crate) task: TaskRef<'db, TMP>,
}

mod impls;
