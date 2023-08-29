use pmrcore::{
    exposure::ExposureRef,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use pmrrepo::handle::GitHandle;

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
}

mod impls;
