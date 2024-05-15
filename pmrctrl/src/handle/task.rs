use pmrcore::{
    platform::{
        MCPlatform,
        TMPlatform,
    },
    task::TaskRef,
};
use pmrtqs::executor::Executor;

use crate::platform::Platform;

pub struct TaskExecutorCtrl<
    'p,
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
> {
    pub(crate) platform: &'p Platform<MCP, TMP>,
    pub(crate) executor: Executor<'p, TMP>,
}

mod impls;
