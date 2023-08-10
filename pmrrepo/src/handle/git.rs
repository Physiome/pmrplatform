use super::*;
use pmrmodel_base::repo::RemoteInfo;
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

pub struct GitHandleResult<'db, 'repo, P: Platform> {
    pub(super) backend: &'db Backend<'db, P>,
    pub repo: &'repo Repository,
    pub commit: Commit<'repo>,
    pub path: &'repo str,
    pub target: GitResultTarget<'repo>,
    pub workspace: &'repo WorkspaceRef<'db, P>,
}

pub(super) mod error;
mod impls;
pub(super) mod util;
