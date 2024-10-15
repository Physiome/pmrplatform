use pmrcore::platform::{
    MCPlatform,
    TMPlatform,
};
use pmrrepo::backend::Backend;
use std::{
    fmt,
    path::{
        Path,
        PathBuf,
    },
    sync::Arc,
};

use crate::platform::Platform;

impl Platform {
    pub fn new(
        ac_platform: pmrac::Platform,
        mc_platform: impl MCPlatform + 'static,
        tm_platform: impl TMPlatform + 'static,
        data_root: PathBuf,
        repo_root: PathBuf,
    ) -> Self {
        let mc_platform = Arc::new(mc_platform);
        let tm_platform = Arc::new(tm_platform);
        let repo_backend = Backend::new(mc_platform.clone(), repo_root.clone());
        Self { ac_platform, mc_platform, tm_platform, data_root, repo_root, repo_backend }
    }

    pub fn data_root(&self) -> &Path {
        self.data_root.as_ref()
    }

    pub fn repo_root(&self) -> &Path {
        self.repo_root.as_ref()
    }

    pub fn repo_backend(&self) -> &Backend {
        &self.repo_backend
    }
}

impl fmt::Debug for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Platform")
            .field("data_root", &self.data_root)
            .field("repo_root", &self.repo_root)
            .finish()
    }
}

mod exposure;
mod profile;
mod task;
