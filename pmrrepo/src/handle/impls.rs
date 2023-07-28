use pmrmodel_base::{
    error::BackendError,
    platform::Platform,
    workspace::{
        WorkspaceRef,
        traits::Workspace,
    },
};
use std::path::PathBuf;

use crate::backend::Backend;
use super::HandleW;

impl<'a, P: Platform + Sync> HandleW<'a, P> {
    pub fn new(
        backend: &'a Backend<P>,
        repo_root: PathBuf,
        workspace: WorkspaceRef<'a, P>,
    ) -> Self {
        Self {
            backend: &backend,
            repo_dir: repo_root.join(workspace.id().to_string()),
            workspace: workspace,
        }
    }
}
