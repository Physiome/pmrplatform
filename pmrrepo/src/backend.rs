use pmrcore::platform::MCPlatform;
use std::path::PathBuf;

use crate::{
    error::PmrRepoError,
    handle::{
        Handle,
        GitHandle,
    },
};

pub struct Backend<'a, P: MCPlatform + Sync> {
    pub(crate) db_platform: &'a P,
    pub(crate) repo_root: PathBuf,
}

impl<'a, P: MCPlatform + Sync> Backend<'a, P> {
    pub fn new(db_platform: &'a P, repo_root: PathBuf) -> Self {
        Self {
            db_platform,
            repo_root,
        }
    }

    pub async fn sync_workspace(&self, workspace_id: i64) -> Result<(), PmrRepoError> {
        let workspace = self.db_platform.get_workspace(workspace_id).await?;
        let handle = Handle::new(&self, self.repo_root.clone(), workspace);
        handle.sync_workspace().await?;
        Ok(())
    }

    pub async fn git_handle(&'a self, workspace_id: i64) -> Result<GitHandle<'a, P>, PmrRepoError> {
        let workspace = self.db_platform.get_workspace(workspace_id).await?;
        Ok(GitHandle::new(&self, self.repo_root.clone(), workspace)?)
    }
}
