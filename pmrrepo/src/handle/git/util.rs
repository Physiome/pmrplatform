use gix::{
    Commit,
    Object,
    Repository,
    actor::SignatureRef,
    object::Kind,
    objs::tree::EntryKind,
    traverse::tree::Recorder,
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
use super::error::FetchClone;

pub(super) fn rev_parse_single<'a>(
    repo: &'a Repository,
    commit_id: &'a str,
) -> Result<Object<'a>, GixError> {
    Ok(repo.rev_parse_single(commit_id)?.object()?)
}

pub(super) fn format_signature_ref(
    value: &SignatureRef,
) -> String {
    format!("{} <{}>", value.name, value.email)
}

pub(super) fn name_from_signature_ref(
    value: &SignatureRef,
) -> String {
    value.name.to_string()
}

pub(super) struct PathFilter<'a> {
    repo: &'a Repository,
    path: Option<&'a str>,
}

impl<'a> PathFilter<'a> {
    pub(super) fn new(
        repo: &'a Repository,
        path: Option<&'a str>,
    ) -> Self {
        PathFilter {
            repo: repo,
            path: path,
        }
    }

    pub(super) fn check(
        &mut self,
        info: &gix::revision::walk::Info,
    ) -> bool {
        self.path
            .map(|path| {
                let oid = self.repo
                    .rev_parse_single(
                        format!("{}:{}", info.id, path).as_str()
                    )
                    .ok();
                // any mismatches will be safe to skip (e.g. when the
                // path does not exist in the commit).
                !info.parent_ids
                    .iter()
                    .all(|id| self.repo
                        .rev_parse_single(
                            format!("{}:{}", id, path).as_str()
                        ).ok() == oid
                    )
            })
            .unwrap_or(true)
    }
}

// TODO there needs to be a way to fully disambiguate commit_id from all
// other strings, so this should map to the underlying Oid for any input
// that isn't None.
pub(crate) fn get_commit<'a>(
    repo: &'a Repository,
    workspace_id: i64,
    commit_id: Option<&'a str>,
) -> Result<Option<Commit<'a>>, PmrRepoError> {
    let obj = match commit_id {
        Some(commit_id) => Some(rev_parse_single(repo, commit_id)?),
        None => rev_parse_single(repo, "HEAD").ok(),
    };
    obj.map(|obj| match obj.kind {
        kind if kind == Kind::Commit => {
            info!("Found {} {}", kind, obj.id);
            match obj.try_into_commit() {
                Ok(commit) => Ok(commit),
                Err(obj) => Err(PmrRepoError::from(ExecutionError::Unexpected {
                    workspace_id: workspace_id,
                    msg: format!("gix said oid {:?} was a commit?", obj.id),
                })),
            }
        }
        _ => {
            Err(PmrRepoError::from(PathError::NoSuchCommit {
                workspace_id: workspace_id,
                // FIXME if it's HEAD it should be a variant along the lines of
                // HEAD not leading to a valid commit
                oid: commit_id.unwrap_or("HEAD").into(),
            }))
        }
    }).transpose()
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
        .peel_to_entry_by_path(
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
) -> Result<(), FetchClone> {
    // using libgit2 as mature protocol support is desired.
    info!("Syncing local {repo_dir:?} with remote <{remote_url}>...");
    let repo_check = git2::Repository::open_bare(&repo_dir);
    match repo_check {
        Ok(repo) => {
            info!("Found existing repo at {repo_dir:?}, synchronizing...");
            let mut remote = repo.find_remote("origin")
                .map_err(|e| FetchClone::Libgit2(e))?;
            match remote.fetch(&[] as &[&str], None, None) {
                Ok(_) => info!("Repository synchronized"),
                Err(e) => return Err(FetchClone::Message(e.to_string())),
            };
        }
        Err(ref e) if e.class() == git2::ErrorClass::Repository => {
            return Err(FetchClone::Message(
                "expected repo_dir be a bare repo".to_string()
            ));
        }
        Err(_) => {
            info!("Cloning new repository at {repo_dir:?}...");
            let mut builder = git2::build::RepoBuilder::new();
            builder.bare(true);
            match builder.clone(remote_url, &repo_dir) {
                Ok(_) => info!("Repository cloned"),
                Err(e) => return Err(FetchClone::Message(
                    format!("fail to clone: {e}")
                )),
            };
        }
    }
    Ok(())
}

pub(super) fn files(
    commit: &Commit<'_>,
) -> Result<Vec<String>, PmrRepoError> {
    let tree = commit.tree().map_err(GixError::from)?;
    let mut recorder = Recorder::default();
    tree.traverse()
        .breadthfirst(&mut recorder).map_err(GixError::from)?;
    let mut results = recorder.records.iter()
        .filter(|entry| entry.mode != EntryKind::Tree.into())
        .filter_map(
            |entry| std::str::from_utf8(entry.filepath.as_ref()).ok()
        )
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    results.sort();
    Ok(results)
}

pub(crate) fn checkout(
    repo: &Repository,
    commit: &Commit<'_>,
    dest_dir: &Path,
) -> Result<(), PmrRepoError> {
    let git_dir = repo.path();
    let odb = gix::odb::at(git_dir.join("objects"))?
        .into_inner()
        .into_arc()?;
    let mut index = gix::index::State::from_tree(
        &commit
            .tree().map_err(GixError::from)?
            .id(),
        &odb
    ).map_err(GixError::from)?;
    let opts = gix::worktree::state::checkout::Options {
        fs: gix::fs::Capabilities::probe(dest_dir),
        destination_is_initially_empty: true,
        .. Default::default()
    };
    gix::worktree::state::checkout(
        &mut index,
        dest_dir,
        odb,
        &gix::features::progress::Discard,
        &gix::features::progress::Discard,
        &std::sync::atomic::AtomicBool::default(),
        opts
    ).map_err(GixError::from)?;
    Ok(())
}
