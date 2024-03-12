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
    sync::OnceLock,
};

use crate::platform::Platform;

impl<
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> Platform<'db, MCP, TMP> {
    pub fn new(
        mc_platform: MCP,
        tm_platform: TMP,
        data_root: PathBuf,
        repo_root: PathBuf,
    ) -> Self {
        let repo_backend = OnceLock::new();
        Self { mc_platform, tm_platform, data_root, repo_root, repo_backend }
    }

    pub fn data_root(&self) -> &Path {
        self.data_root.as_ref()
    }

    pub fn repo_backend<'p>(
        &'p self
    ) -> &'p Backend<'db, MCP>
    where
        'p: 'db
    {
        match self.repo_backend.get() {
            Some(repo_backend) => repo_backend,
            None => {
                self.repo_backend.set(
                    Backend::new(&self.mc_platform, self.repo_root.clone())
                ).unwrap_or_else(|_| log::warn!(
                    "duplicate call to repo_backend while it is being setup"
                ));
                self.repo_backend.get()
                    .expect("this repo_backend just got set!")
            }
        }
    }
}

mod exposure;
mod profile;
mod task;
