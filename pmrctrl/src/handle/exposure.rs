use pmrcore::{
    exposure::Exposure,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use pmrrepo::handle::GitHandle;

use crate::platform::Platform;

pub struct ExposureCtrl<
    'a,
    MCP: MCPlatform + Sync,
    TMP: TMPlatform + Sync,
> {
    pub(crate) platform: &'a Platform<'a, MCP, TMP>,
    // TODO maybe this could also follow the OnceLock pattern for on-demand
    // usage?
    pub(crate) git_handle: GitHandle<'a, MCP>,
    pub(crate) inner: Exposure,
}
