use async_trait::async_trait;
use pmrcore::{
    platform::{
        MCPlatform,
        TMPlatform,
    },
};

use crate::{
    handle::TaskCtrl,
    error::PlatformError,
};

impl<
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> TaskCtrl<'db, MCP, TMP> {
}
