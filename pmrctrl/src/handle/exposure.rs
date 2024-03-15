use parking_lot::Mutex;
use pmrcore::{
    exposure::{
        ExposureRef,
        ExposureFileRef,
    },
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
    // TODO need a workspace loader?
    //      - the platform does provide a root, this can facilitate the copy
    //        to disk method
    //      - for the pmrgit-fuse version, that may provide a mount point
    //      - perhaps also provide this via a conversion trait?
    //      - an intermediate WorkspaceCtrl or WorkspaceCheckoutCtrl of some
    //        form will be very useful; this may be done via a trait so that
    //        both copy/fuse version can be swapped into place?
}

pub struct ExposureCtrl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
>(pub(crate) Arc<RawExposureCtrl<'p, 'db, MCP, TMP>>);

mod impls;
