use super::*;
use pmrcore::{
    git::PathObjectDetached,
    repo::RemoteInfo,
};
use gix::{
    ObjectDetached,
    ThreadSafeRepository,
};

pub struct GitHandle<'db, 'repo, P: MCPlatform + Sync> {
    pub(super) backend: &'repo Backend<'db, P>,
    pub(super) workspace: WorkspaceRef<'db, P>,
    pub(super) repo: ThreadSafeRepository,
}

#[derive(Debug)]
pub enum GitResultTarget {
    Object(PathObjectDetached),
    RemoteInfo(RemoteInfo),
}

pub struct GitHandleResult<'db, 'repo, P: MCPlatform + Sync> {
    pub(super) backend: &'repo Backend<'db, P>,
    pub(super) repo: &'repo ThreadSafeRepository,
    pub(super) commit: ObjectDetached,
    pub(super) target: GitResultTarget,
    pub(super) workspace: &'repo WorkspaceRef<'db, P>,
}

pub(super) mod error;
mod impls;
pub(super) mod util;
