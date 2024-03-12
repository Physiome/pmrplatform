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
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> TaskCtrl<'p, 'db, MCP, TMP> {
}
