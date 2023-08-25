use pmrcore::{
    exposure::ExposureFile,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use pmrrepo::handle::GitHandleResult;

use crate::platform::Platform;

pub struct ExposureFileCtrl<
    'db,
    'repo,
    MCP: MCPlatform + Sync,
    TMP: TMPlatform + Sync,
> {
    pub(crate) platform: &'db Platform<'db, MCP, TMP>,
    // TODO maybe this could also follow the OnceLock pattern for on-demand
    // usage?
    pub(crate) pathinfo: GitHandleResult<'db, 'repo, MCP>,
    pub(crate) inner: ExposureFile,
}

mod impls;
