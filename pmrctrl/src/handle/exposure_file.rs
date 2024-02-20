use parking_lot::MappedMutexGuard;
use pmrcore::{
    exposure::ExposureFileRef,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use pmrrepo::handle::GitHandleResult;

use crate::platform::Platform;

pub struct ExposureFileCtrl<
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    pub(crate) platform: &'db Platform<'db, MCP, TMP>,
    // Given that the GitHandleResult in this struct contains things
    // typically owned by the GitHandle inside the ExposureCtrl that
    // spawned this, it makes sense to also have this owned by that
    pub(crate) exposure_file: MappedMutexGuard<'db, ExposureFileRef<'db, MCP>>,
    pub(crate) pathinfo: GitHandleResult<'db, 'db, MCP>,
}

mod impls;
