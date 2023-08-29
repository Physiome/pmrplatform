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
    'repo,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    pub(crate) platform: &'db Platform<'db, MCP, TMP>,
    // TODO maybe this could also follow the OnceLock pattern for on-demand
    // usage?
    pub(crate) pathinfo: GitHandleResult<'db, 'repo, MCP>,
    pub exposure_file: ExposureFileRef<'db, MCP>,
}

mod impls;
