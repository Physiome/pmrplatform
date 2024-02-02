use pmrcore::platform::{
    MCPlatform,
    TMPlatform,
};
use pmrrepo::backend::Backend;
use std::{
    path::PathBuf,
    sync::OnceLock,
};

pub struct Platform<
    'a,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    pub mc_platform: MCP,
    pub tm_platform: TMP,
    pub(crate) data_root: PathBuf,
    pub(crate) repo_root: PathBuf,
    pub(crate) repo_backend: OnceLock<Backend<'a, MCP>>,
}
