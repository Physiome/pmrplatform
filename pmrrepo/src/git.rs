pub use gix::object::Kind;
use gix::{
    Commit,
    Object,
    Repository,
};
use std::path::Path;
use crate::{
    error::{
        ContentError,
        ExecutionError,
        GixError,
        PathError,
        PmrRepoError,
    },
};

mod impls;
pub(crate) mod util;
use util::*;

pub use impls::{
    HandleW,
    HandleWR,
    WorkspaceGitResult,
    stream_git_result_default,
    stream_git_result_as_json,
    stream_blob,
};

pub enum GitResultTarget<'a> {
    Object(Object<'a>),
    SubRepoPath {
        location: String,
        commit: String,
        path: &'a str,
    },
}

pub struct GitResult<'a> {
    pub repo: &'a Repository,
    pub commit: Commit<'a>,
    pub path: &'a str,
    pub target: GitResultTarget<'a>,
}

pub(crate) fn get_commit<'a>(
    repo: &'a Repository,
    workspace_id: i64,
    commit_id: Option<&'a str>,
) -> Result<Commit<'a>, PmrRepoError> {
    let obj = rev_parse_single(repo, &commit_id.unwrap_or("HEAD"))?;
    match obj.kind {
        kind if kind == Kind::Commit => {
            info!("Found {} {}", kind, obj.id);
        }
        _ => return Err(PathError::NoSuchCommit {
            workspace_id: workspace_id,
            oid: commit_id.unwrap_or("HEAD?").into(),
        }.into())
    }
    match obj.try_into_commit() {
        Ok(commit) => Ok(commit),
        Err(obj) => Err(ExecutionError::Unexpected {
            workspace_id: workspace_id,
            msg: format!("gix said oid {:?} was a commit?", obj.id),
        }.into()),
    }
}

pub(crate) fn get_submodule_target(
    commit: &Commit,
    workspace_id: i64,
    path: &str,
) -> Result<String, PmrRepoError> {
    let blob = commit
        .tree_id().map_err(GixError::from)?
        .object().map_err(GixError::from)?
        .try_into_tree().map_err(GixError::from)?
        .lookup_entry_by_path(
            Path::new(".gitmodules")).map_err(GixError::from)?
        .ok_or_else(|| PmrRepoError::from(PathError::NoSuchPath {
            workspace_id: workspace_id,
            oid: commit.id.to_string(),
            path: path.to_string(),
        }))?
        .id()
        .object().map_err(GixError::from)?;
    let config = gix::config::File::try_from(
        std::str::from_utf8(&blob.data)
        .map_err(
            |e| PmrRepoError::from(ContentError::Invalid {
                workspace_id: workspace_id,
                oid: commit.id().to_string(),
                path: path.to_string(),
                msg: format!("error parsing `.gitmodules`: {}", e),
            })
        )?
    ).map_err(
        |e| PmrRepoError::from(ContentError::Invalid {
            workspace_id: workspace_id,
            oid: commit.id().to_string(),
            path: path.to_string(),
            msg: format!("error parsing `.gitmodules`: {}", e),
        })
    )?;
    for rec in config.sections_and_ids() {
        match rec.0.value("path") {
            Some(rec_path) => {
                if path == rec_path.into_owned() {
                    return Ok(format!("{}", rec.0.value("url").unwrap()));
                }
            },
            None => {},
        }
    }
    Err(PathError::NotSubmodule {
        workspace_id: workspace_id,
        oid: commit.id().to_string(),
        path: path.to_string(),
    }.into())
}

pub(crate) fn fetch_or_clone(
    repo_dir: &Path,
    remote_url: &str,
) -> Result<(), error::FetchClone> {
    // using libgit2 as mature protocol support is desired.
    info!("Syncing local {repo_dir:?} with remote <{remote_url}>...");
    let repo_check = git2::Repository::open_bare(&repo_dir);
    match repo_check {
        Ok(repo) => {
            info!("Found existing repo at {repo_dir:?}, synchronizing...");
            let mut remote = repo.find_remote("origin")
                .map_err(|e| error::FetchClone::Libgit2(e))?;
            match remote.fetch(&[] as &[&str], None, None) {
                Ok(_) => info!("Repository synchronized"),
                Err(e) => return Err(error::FetchClone::Message(e.to_string())),
            };
        }
        Err(ref e) if e.class() == git2::ErrorClass::Repository => {
            return Err(error::FetchClone::Message(
                "expected repo_dir be a bare repo".to_string()
            ));
        }
        Err(_) => {
            info!("Cloning new repository at {repo_dir:?}...");
            let mut builder = git2::build::RepoBuilder::new();
            builder.bare(true);
            match builder.clone(remote_url, &repo_dir) {
                Ok(_) => info!("Repository cloned"),
                Err(e) => return Err(error::FetchClone::Message(
                    format!("fail to clone: {e}")
                )),
            };
        }
    }
    Ok(())
}

pub(crate) mod error {
    pub(crate) enum FetchClone {
        Libgit2(git2::Error),
        Message(String),
    }
}
