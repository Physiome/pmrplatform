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

use crate::platform::Platform;

pub struct ExposureCtrl<
    'a,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    pub(crate) platform: &'a Platform<'a, MCP, TMP>,
    // TODO maybe this could also follow the OnceLock pattern for on-demand
    // usage?
    pub(crate) git_handle: GitHandle<'a, MCP>,
    pub exposure: ExposureRef<'a, MCP>,
    pub(crate) exposure_files: Arc<Mutex<HashMap<&'a str, ExposureFileRef<'a, MCP>>>>,
    // TODO need a workspace loader?
    //      - the platform does provide a root, this can facilitate the copy
    //        to disk method
    //      - for the pmrgit-fuse version, that may provide a mount point
    //      - perhaps also provide this via a conversion trait?
    //      - an intermediate WorkspaceCtrl or WorkspaceCheckoutCtrl of some
    //        form will be very useful; this may be done via a trait so that
    //        both copy/fuse version can be swapped into place?
}

mod impls;
