use pmrcore::{
    exposure::ExposureFileRef,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use pmrrepo::handle::GitHandleResult;
use std::sync::Arc;

use crate::{
    platform::Platform,
    handle::ExposureCtrl,
};

pub struct ExposureFileCtrl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    pub(crate) platform: &'p Platform<'db, MCP, TMP>,
    pub(crate) exposure: &'p ExposureCtrl<'db, 'db, MCP, TMP>,
    pub(crate) exposure_file: ExposureFileRef<'db, MCP>,
    pub(crate) pathinfo: GitHandleResult<'p, 'db, MCP>,
}

mod impls;
