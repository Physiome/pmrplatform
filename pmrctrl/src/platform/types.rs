use pmrcore::platform::{
    MCPlatform,
    TMPlatform,
};
use pmrrepo::backend::Backend;
use std::{
    path::PathBuf,
    sync::Arc,
};

pub struct Platform<
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
> {
    pub mc_platform: Arc<MCP>,
    pub tm_platform: Arc<TMP>,
    pub(crate) data_root: PathBuf,
    pub(crate) repo_root: PathBuf,
    pub(crate) repo_backend: Backend<MCP>,
}
