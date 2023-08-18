use super::*;
use pmrcore::{
    git::PathObject,
    repo::RemoteInfo,
};
use gix::{
    Commit,
    Repository,
};

pub struct GitHandle<'a, P: Platform> {
    pub(super) backend: &'a Backend<'a, P>,
    pub workspace: WorkspaceRef<'a, P>,
    pub repo: Repository,
}

#[derive(Debug)]
pub enum GitResultTarget<'a> {
    Object(PathObject<'a>),
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
