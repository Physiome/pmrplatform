use pmrcore::{
    exposure::{
        ExposureFileView,
        traits::ExposureFileViewBackend,
    },
    platform::{
        MCPlatform,
        TMPlatform,
    },
};

use super::ExposureFileViewCtrl;
use crate::{
    error::PlatformError,
};

impl<
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> ExposureFileViewCtrl<'db, MCP, TMP> {
}
