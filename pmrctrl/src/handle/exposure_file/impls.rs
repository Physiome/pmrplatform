use pmrcore::{
    exposure::traits::ExposureFileBackend,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};

use super::ExposureFileCtrl;
use crate::{
    error::PlatformError,
};

impl<
    'db,
    'repo,
    MCP: MCPlatform + Sync,
    TMP: TMPlatform + Sync,
> ExposureFileCtrl<'db, 'repo, MCP, TMP> {
}
