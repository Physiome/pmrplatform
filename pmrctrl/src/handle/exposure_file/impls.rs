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
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> ExposureFileCtrl<'db, 'repo, MCP, TMP> {
}
