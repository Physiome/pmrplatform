use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PmrRepoError {
    #[error("BackendError: {0}")]
    BackendError(#[from] pmrcore::error::BackendError),
    #[error("ContentError: {0}")]
    ContentError(#[from] ContentError),
    #[error("ExecutionError: {0}")]
    ExecutionError(#[from] ExecutionError),
    #[error("GixError: {0}")]
    GixError(#[from] GixError),
    #[error("Libgit2Error: {0}")]
    Libgit2Error(#[from] git2::Error),
    #[error("PathError: {0}")]
    PathError(#[from] PathError),
    #[error("StdIoError: {0}")]
    StdIoError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum GixError {
    #[error(transparent)]
    HashDecode(#[from] gix::hash::decode::Error),
    #[error(transparent)]
    IndexFileInit(#[from] gix::index::file::init::Error),
    #[error(transparent)]
    ObjectCommit(#[from] gix::object::commit::Error),
    #[error(transparent)]
    ObjectDecode(#[from] gix::objs::decode::Error),
    #[error(transparent)]
    ObjectTryInto(#[from] gix::object::try_into::Error),
    #[error(transparent)]
    OdbFindExisting(#[from] gix::object::find::existing::Error),
    #[error(transparent)]
    Open(#[from] gix::open::Error),
    #[error(transparent)]
    ReferenceIter(#[from] gix::reference::iter::Error),
    #[error(transparent)]
    ReferenceIterInit(#[from] gix::reference::iter::init::Error),
    #[error(transparent)]
    RevisionSpecParseSingle(#[from] gix::revision::spec::parse::single::Error),
    #[error(transparent)]
    RevisionWalk(#[from] gix::revision::walk::Error),
    #[error(transparent)]
    TraverseCommitAncestors(#[from] gix::traverse::commit::ancestors::Error),
    #[error(transparent)]
    TraverseTreeBreadthfirst(#[from] gix::traverse::tree::breadthfirst::Error),
    #[error(transparent)]
    WorktreeStateCheckout(#[from] gix::worktree::state::checkout::Error),
}

#[derive(Debug, PartialEq, Error, Deserialize, Serialize)]
pub enum ContentError {
    #[error("workspace `{workspace_id}` at commit `{oid}` at path `{path}` \n\
             provided invalid content: {msg}")]
    Invalid {
        workspace_id: i64,
        oid: String,
        path: String,
        msg: String,
    },
    #[error("workspace `{workspace_id}` references a repository at `{url}` \n\
             but it is not currently registered as a workspace")]
    NoWorkspaceForUrl {
        workspace_id: i64,
        url: String,
    },
}

#[derive(Debug, PartialEq, Error, Deserialize, Serialize)]
pub enum ExecutionError {
    #[error("workspace `{workspace_id}`: failed to synchronize with \
             remote `{remote}`: {msg}")]
    Synchronize {
        workspace_id: i64,
        remote: String,
        msg: String,
    },
    #[error("workspace `{workspace_id}`: unexpected error: {msg}")]
    Unexpected {
        workspace_id: i64,
        msg: String,
    }
}

#[derive(Debug, PartialEq, Error, Deserialize, Serialize)]
pub enum PathError {
    #[error("workspace `{workspace_id}` at commit `{oid}`: not \n\
             a submodule at `{path}`")]
    NotSubmodule {
        workspace_id: i64,
        oid: String,
        path: String,
    },
    #[error("workspace `{workspace_id}`: no commit `{oid}`")]
    NoSuchCommit {
        workspace_id: i64,
        oid: String,
    },
    #[error("workspace `{workspace_id}` at commit `{oid}`: \
             no such path {path}")]
    NoSuchPath {
        workspace_id: i64,
        oid: String,
        path: String,
    },
    #[error("couldn't open repository for workspace `{workspace_id}`")]
    Repository {
        workspace_id: i64,
    },
}
