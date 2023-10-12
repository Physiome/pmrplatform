use pmrcore::{
    platform::{
        MCPlatform,
        TMPlatform,
    },
    task::Task,
};

use crate::platform::Platform;

pub struct TaskCtrl<
    'a,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    pub(crate) platform: &'a Platform<'a, MCP, TMP>,
    pub(crate) task: Task,
}

mod impls;
