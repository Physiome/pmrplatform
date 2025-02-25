use pmrcore::platform::MCPlatform;
use std::{
    path::PathBuf,
    sync::Arc,
};

use crate::{
    error::PmrRepoError,
    handle::{
        Handle,
        GitHandle,
    },
};

#[derive(Clone)]
pub struct Backend {
    pub db_platform: Arc<dyn MCPlatform + Send + Sync>,
    pub(crate) repo_root: PathBuf,
}

impl Backend {
    pub fn new(db_platform: Arc<dyn MCPlatform + Send + Sync>, repo_root: PathBuf) -> Self {
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

    pub async fn git_handle<'a>(&'a self, workspace_id: i64) -> Result<GitHandle<'a>, PmrRepoError> {
        let workspace = self.db_platform.get_workspace(workspace_id).await?;
        Ok(GitHandle::new(&self, self.repo_root.clone(), workspace))
    }

    pub fn platform(&self) -> &(dyn MCPlatform + Send + Sync) {
        self.db_platform.as_ref()
    }
}
