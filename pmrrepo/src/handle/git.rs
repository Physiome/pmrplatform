use super::*;
use pmrcore::{
    git::PathObject,
    repo::RemoteInfo,
};
use gix::{
    Commit,
    Repository,
};

pub struct GitHandle<'db, 'repo, P: MCPlatform + Sync> {
    pub(super) backend: &'repo Backend<'db, P>,
    pub(super) workspace: WorkspaceRef<'db, P>,
    pub(super) repo: Repository,
}

#[derive(Debug)]
pub enum GitResultTarget<'a> {
    Object(PathObject<'a>),
    RemoteInfo(RemoteInfo),
}

pub struct GitHandleResult<'db, 'repo, P: MCPlatform + Sync> {
    pub(super) backend: &'repo Backend<'db, P>,
    pub(super) repo: &'repo Repository,
    pub(super) commit: Commit<'repo>,
    pub(super) target: GitResultTarget<'repo>,
    pub(super) workspace: &'repo WorkspaceRef<'db, P>,
}

pub(super) mod error;
mod impls;
pub(super) mod util;
