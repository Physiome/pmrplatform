use pmrmodel_base::{
    platform::Platform,
    workspace::WorkspaceRef,
};
use std::path::PathBuf;

use crate::{
    error::PmrRepoError,
    handle::{
        HandleW,
        HandleWR,
    },
};

pub struct Backend<'a, P: Platform> {
    pub(crate) db_platform: &'a P,
    pub(crate) repo_root: PathBuf,
}

impl<'a, P: Platform + Sync> Backend<'a, P> {
    pub fn new(db_platform: &'a P, repo_root: PathBuf) -> Self {
        Self {
            db_platform,
            repo_root,
        }
    }

    pub async fn sync_workspace(&self, workspace_id: i64) -> Result<(), PmrRepoError> {
        let workspace = self.db_platform.get_workspace(workspace_id).await?;
        let handle = HandleW::new(&self, self.repo_root.clone(), workspace);
        let workspace = handle.sync_workspace().await?;
        let handle = HandleWR::new(&self, self.repo_root.clone(), workspace)?;
        Ok(handle.index_tags().await?)
    }
}
