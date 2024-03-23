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
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
> {
    pub(crate) platform: &'p Platform<MCP, TMP>,
    pub(crate) task: TaskRef<'p, TMP>,
}

mod impls;
