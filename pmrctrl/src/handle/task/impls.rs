use pmrcore::{
    platform::{
        MCPlatform,
        TMPlatform,
    },
};

use crate::{
    handle::TaskCtrl,
    // error::PlatformError,
};

impl<
    'p,
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
> TaskCtrl<'p, MCP, TMP> {
}
