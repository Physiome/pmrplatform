use super::*;
use pmrcore::{
    git::PathObjectDetached,
    repo::RemoteInfo,
};
use gix::{
    ObjectDetached,
    ThreadSafeRepository,
};

pub struct GitHandle<'repo, P: MCPlatform + Send + Sync> {
    pub(super) backend: &'repo Backend<P>,
    pub(super) workspace: WorkspaceRef<'repo, P>,
    pub(super) repo: ThreadSafeRepository,
}

#[derive(Debug)]
pub enum GitResultTarget {
    Object(PathObjectDetached),
    RemoteInfo(RemoteInfo),
}

pub struct GitHandleResult<'repo, P: MCPlatform + Send + Sync> {
    pub(super) backend: &'repo Backend<P>,
    pub(super) repo: &'repo ThreadSafeRepository,
    pub(super) commit: ObjectDetached,
    pub(super) target: GitResultTarget,
    pub(super) workspace: &'repo WorkspaceRef<'repo, P>,
}

pub(super) mod error;
mod impls;
pub(super) mod util;
