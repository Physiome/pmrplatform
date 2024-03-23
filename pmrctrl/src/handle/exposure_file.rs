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
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
> {
    pub(crate) platform: &'p Platform<MCP, TMP>,
    pub(crate) exposure: ExposureCtrl<'p, MCP, TMP>,
    pub(crate) exposure_file: ExposureFileRef<'p, MCP>,
    pub(crate) pathinfo: GitHandleResult<'p, MCP>,
    data_root: PathBuf,
}

pub struct ExposureFileCtrl<
    'p,
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
>(pub(crate) Arc<RawExposureFileCtrl<'p, MCP, TMP>>);

mod impls;
