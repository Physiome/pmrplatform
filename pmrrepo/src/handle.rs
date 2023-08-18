use pmrcore::{
    platform::MCPlatform,
    workspace::WorkspaceRef,
};
use std::path::PathBuf;
use crate::backend::Backend;

pub(crate) struct Handle<'a, P: MCPlatform> {
    backend: &'a Backend<'a, P>,
    pub(crate) repo_dir: PathBuf,
    pub(crate) workspace: WorkspaceRef<'a, P>,
}

mod impls;
mod git;

pub use git::{
    GitHandle,
    GitResultTarget,
    GitHandleResult,
};
