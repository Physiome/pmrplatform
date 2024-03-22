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
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    pub(crate) platform: &'p Platform<'db, MCP, TMP>,
    pub(crate) git_handle: GitHandle<'db, 'db, MCP>,
    pub(crate) exposure: ExposureRef<'db, MCP>,
    pub(crate) exposure_file_ctrls: Arc<Mutex<HashMap<String, ExposureFileCtrl<'p, 'db, MCP, TMP>>>>,
}

pub struct ExposureCtrl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
>(pub(crate) Arc<RawExposureCtrl<'p, 'db, MCP, TMP>>);

mod impls;
