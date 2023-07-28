use pmrmodel_base::{
    error::BackendError,
    platform::Platform,
};
use std::path::PathBuf;

use crate::handle::HandleW;

pub struct Backend<P: Platform> {
    db_platform: P,
    repo_root: PathBuf,
}

impl<P: Platform + Sync> Backend<P> {
    pub fn new(db_platform: P, repo_root: PathBuf) -> Self {
        Self {
            db_platform,
            repo_root,
        }
    }

    pub async fn handle_w(&self, workspace_id: i64) -> Result<HandleW<P>, BackendError> {
        let workspace = self.db_platform.get_workspace(workspace_id).await?;
        Ok(HandleW::new(&self, self.repo_root.clone(), workspace))
    }
}
