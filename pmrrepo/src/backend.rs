use pmrcore::platform::MCPlatform;
use std::path::PathBuf;

use crate::{
    error::PmrRepoError,
    handle::{
        Handle,
        GitHandle,
    },
};

pub struct Backend<'db, P: MCPlatform + Sync> {
    pub(crate) db_platform: &'db P,
    pub(crate) repo_root: PathBuf,
}

impl<'db, 'repo, P: MCPlatform + Sync> Backend<'db, P> {
    pub fn new(db_platform: &'db P, repo_root: PathBuf) -> Self {
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

    pub async fn git_handle(
        &'repo self,
        workspace_id: i64,
    ) -> Result<GitHandle<'repo, 'db, P>, PmrRepoError>
    where
        'repo: 'db
    {
        let workspace = self.db_platform.get_workspace(workspace_id).await?;
        Ok(GitHandle::new(&self, self.repo_root.clone(), workspace)?)
    }
}
