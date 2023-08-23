use pmrcore::platform::{
    MCPlatform,
    TMPlatform,
};
use pmrrepo::backend::Backend;
use std::{
    path::PathBuf,
    sync::OnceLock,
};

use crate::platform::Platform;

impl<
    'a,
    MCP: MCPlatform + Sync,
    TMP: TMPlatform + Sync,
> Platform<'a, MCP, TMP> {
    pub fn new(mc_platform: MCP, tm_platform: TMP, repo_root: PathBuf) -> Self {
        let repo_backend = OnceLock::new();
        Self { mc_platform, tm_platform, repo_root, repo_backend }
    }

    pub fn repo_backend(
        &'a self
    ) -> &'a Backend<'a, MCP> {
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
