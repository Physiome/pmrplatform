use parking_lot::MappedMutexGuard;
use pmrcore::{
    exposure::ExposureFileRef,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use pmrrepo::handle::GitHandleResult;

use crate::{
    platform::Platform,
    handle::ExposureCtrl,
};

pub(crate) struct EFCData<
    'a,
    MCP: MCPlatform + Sized + Sync,
> {
    pub(crate) exposure_file: ExposureFileRef<'a, MCP>,
    pub(crate) pathinfo: GitHandleResult<'a, 'a, MCP>,
}

pub struct ExposureFileCtrl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    pub(crate) platform: &'p Platform<'db, MCP, TMP>,
    pub(crate) exposure: &'p ExposureCtrl<'db, 'db, MCP, TMP>,
    pub(crate) data: MappedMutexGuard<'db, EFCData<'db, MCP>>,
}

mod impls;
