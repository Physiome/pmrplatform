use super::*;
use pmrcore::{
    git::PathObjectDetached,
    repo::RemoteInfo,
};
use gix::{
    ObjectDetached,
    ThreadSafeRepository,
};
use std::{
    path::PathBuf,
    sync::OnceLock,
};
use crate::error::GixError;

pub struct GitHandle<'repo> {
    pub(super) backend: &'repo Backend,
    pub(super) workspace: WorkspaceRef<'repo>,
    pub(super) repo_dir: PathBuf,
    pub(super) repo: OnceLock<Result<ThreadSafeRepository, GixError>>,
}

#[derive(Debug)]
pub enum GitResultTarget {
    Object(PathObjectDetached),
    RemoteInfo(RemoteInfo),
}

pub struct GitHandleResult<'repo> {
    pub(super) backend: &'repo Backend,
    pub(super) repo: &'repo ThreadSafeRepository,
    pub(super) commit: Option<ObjectDetached>,
    pub(super) target: Option<GitResultTarget>,
    pub(super) workspace: &'repo WorkspaceRef<'repo>,
}

pub(super) mod error;
mod impls;
pub(super) mod util;
