use parking_lot::Mutex;
use pmrcore::{
    exposure::ExposureRef,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use pmrrepo::handle::GitHandle;
use std::{
    collections::HashMap,
    sync::Arc,
};

use crate::{
    platform::Platform,
    handle::ExposureFileCtrl,
};

pub(crate) struct RawExposureCtrl<
    'p,
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
> {
    pub(crate) platform: &'p Platform<MCP, TMP>,
    pub(crate) git_handle: GitHandle<'p, MCP>,
    pub(crate) exposure: ExposureRef<'p, MCP>,
    pub(crate) exposure_file_ctrls: Arc<Mutex<HashMap<String, ExposureFileCtrl<'p, MCP, TMP>>>>,
}

pub struct ExposureCtrl<
    'p,
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
>(pub(crate) Arc<RawExposureCtrl<'p, MCP, TMP>>);

mod impls;
