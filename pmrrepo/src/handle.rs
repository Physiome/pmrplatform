use gix::Repository;
use pmrmodel_base::{
    platform::Platform,
    workspace::WorkspaceRef,
};
use std::path::PathBuf;
use crate::backend::Backend;

pub(crate) struct HandleW<'a, P: Platform> {
    backend: &'a Backend<'a, P>,
    pub(crate) repo_dir: PathBuf,
    pub(crate) workspace: WorkspaceRef<'a, P>,
}

pub struct HandleWR<'a, P: Platform> {
    backend: &'a Backend<'a, P>,
    pub(crate) repo_dir: PathBuf,
    pub(crate) workspace: WorkspaceRef<'a, P>,
    pub(crate) repo: Repository,
}

mod impls;
