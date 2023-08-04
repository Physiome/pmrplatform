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
    pub(crate) workspace: WorkspaceRef<'a, P>,
    pub(crate) repo: Repository,
}

pub enum GitResultTarget<'a> {
    Object(Object<'a>),
    RemoteInfo(RemoteInfo),
}

pub struct GitHandleResult<'a, P: Platform> {
    pub(super) backend: &'a Backend<'a, P>,
    pub repo: &'a Repository,
    pub commit: Commit<'a>,
    pub path: &'a str,
    pub target: GitResultTarget<'a>,
    pub workspace: &'a WorkspaceRef<'a, P>,
}
