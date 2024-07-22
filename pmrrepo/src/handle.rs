use pmrcore::workspace::WorkspaceRef;
use std::path::PathBuf;
use crate::backend::Backend;

pub(crate) struct Handle<'db> {
    backend: &'db Backend,
    pub(crate) repo_dir: PathBuf,
    pub(crate) workspace: WorkspaceRef<'db>,
}

mod impls;
mod git;

pub use git::{
    GitHandle,
    GitResultTarget,
    GitHandleResult,
};
