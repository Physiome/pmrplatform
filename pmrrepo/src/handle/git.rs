use super::*;
use pmrmodel_base::git::RemoteInfo;
use gix::{
    Commit,
    Object,
    Repository,
};

pub struct GitHandle<'a, P: Platform> {
    pub(super) backend: &'a Backend<'a, P>,
    pub(crate) repo_dir: PathBuf,
    pub workspace: WorkspaceRef<'a, P>,
    pub repo: Repository,
}

pub enum GitResultTarget<'a> {
    Object(Object<'a>),
    RemoteInfo(RemoteInfo),
}

pub struct GitHandleResult<'a, 'b, P: Platform> {
    pub(super) backend: &'a Backend<'a, P>,
    pub repo: &'b Repository,
    pub commit: Commit<'b>,
    pub path: &'a str,
    pub target: GitResultTarget<'b>,
    pub workspace: &'b WorkspaceRef<'a, P>,
}

pub(super) mod error;
mod impls;
pub(super) mod util;
