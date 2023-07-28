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

mod impls;

pub struct HandleW<'a, P: Platform> {
    backend: &'a Backend<P>,
    repo_dir: PathBuf,
    pub workspace: WorkspaceRef<'a, P>,
}
