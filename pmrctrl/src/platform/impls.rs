use pmrcore::platform::{
    MCPlatform,
    TMPlatform,
};
use pmrrepo::backend::Backend;
use std::{
    path::{
        Path,
        PathBuf,
    },
    sync::Arc,
};

use crate::platform::Platform;

impl<
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
> Platform<MCP, TMP> {
    pub fn new(
        mc_platform: MCP,
        tm_platform: TMP,
        data_root: PathBuf,
        repo_root: PathBuf,
    ) -> Self {
        let mc_platform = Arc::new(mc_platform);
        let tm_platform = Arc::new(tm_platform);
        let repo_backend = Backend::new(mc_platform.clone(), repo_root.clone());
        Self { mc_platform, tm_platform, data_root, repo_root, repo_backend }
    }

    pub fn data_root(&self) -> &Path {
        self.data_root.as_ref()
    }

    pub fn repo_root(&self) -> &Path {
        self.repo_root.as_ref()
    }

    pub fn repo_backend(&self) -> &Backend<MCP> {
        &self.repo_backend
    }
}

mod exposure;
mod profile;
mod task;
