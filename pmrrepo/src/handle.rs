use pmrcore::{
    platform::MCPlatform,
    workspace::WorkspaceRef,
};
use std::path::PathBuf;
use crate::backend::Backend;

pub(crate) struct Handle<'handle, 'db, P: MCPlatform + Sync> {
    backend: &'handle Backend<'db, P>,
    pub(crate) repo_dir: PathBuf,
    pub(crate) workspace: WorkspaceRef<'db, P>,
}

mod impls;
mod git;

pub use git::{
    GitHandle,
    GitResultTarget,
    GitHandleResult,
};
