use pmrcore::{
    exposure::ExposureFileRef,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use pmrrepo::handle::GitHandleResult;
use std::{
    path::PathBuf,
    sync::Arc,
};

use crate::{
    platform::Platform,
    handle::ExposureCtrl,
};

pub(crate) struct RawExposureFileCtrl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    pub(crate) platform: &'p Platform<'db, MCP, TMP>,
    pub(crate) exposure: ExposureCtrl<'db, 'db, MCP, TMP>,
    pub(crate) exposure_file: ExposureFileRef<'db, MCP>,
    pub(crate) pathinfo: GitHandleResult<'p, 'db, MCP>,
    data_root: PathBuf,
}

pub struct ExposureFileCtrl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
>(pub(crate) Arc<RawExposureFileCtrl<'p, 'db, MCP, TMP>>);

mod impls;
