use super::*;
use pmrcore::{
    git::PathObject,
    repo::RemoteInfo,
};
use gix::{
    Commit,
    Repository,
};

pub struct GitHandle<'a, P: MCPlatform + Sync> {
    pub(super) backend: &'a Backend<'a, P>,
    pub(super) workspace: WorkspaceRef<'a, P>,
    pub(super) repo: Repository,
}

#[derive(Debug)]
pub enum GitResultTarget<'a> {
    Object(PathObject<'a>),
    RemoteInfo(RemoteInfo),
}

pub struct GitHandleResult<'db, 'repo, P: MCPlatform + Sync> {
    pub(super) backend: &'db Backend<'db, P>,
    pub(super) repo: &'repo Repository,
    pub(super) commit: Commit<'repo>,
    pub(super) path: String,
    pub(super) target: GitResultTarget<'repo>,
    pub(super) workspace: &'repo WorkspaceRef<'db, P>,
}

pub(super) mod error;
mod impls;
pub(super) mod util;
