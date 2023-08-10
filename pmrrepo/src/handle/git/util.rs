use gix::{
    Commit,
    Object,
    Repository,
    actor::SignatureRef,
    object::Kind,
    objs::{
        BlobRef,
        CommitRef,
        TreeRef,
        WriteTo as _,
    },
};
use pmrmodel_base::repo::{
    CommitInfo,
    FileInfo,
    ObjectInfo,
    TreeEntryInfo,
    TreeInfo,
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
    util::is_binary,
};
use super::{
    Platform,
    GitHandleResult,
    error::FetchClone,
};

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

pub(super) fn get_commit<'a>(
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

pub(super) fn get_submodule_target(
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

// These assume the blobs are all contained because the conversion to
// the Ref equivalent currently drops information for gix, and to make
// the internal usage consistent, the raw object is passed.
pub(super) fn obj_blob_to_info(git_object: &Object, path: Option<&str>) -> ObjectInfo {
    let blob = BlobRef::from_bytes(&git_object.data).unwrap();
    ObjectInfo::FileInfo(FileInfo {
        size: blob.size() as u64,
        binary: is_binary(blob.data),
        mime_type: path
            .and_then(|path| mime_guess::from_path(path).first_raw())
            .unwrap_or("application/octet-stream")
            .to_string(),
    })
}

pub(super) fn obj_tree_to_info(git_object: &Object) -> ObjectInfo {
    let tree = TreeRef::from_bytes(&git_object.data).unwrap();
    ObjectInfo::TreeInfo(
        TreeInfo {
            filecount: tree.entries.len() as u64,
            entries: tree.entries.iter().map(|entry| TreeEntryInfo {
                filemode: std::str::from_utf8(entry.mode.as_bytes()).unwrap().to_string(),
                kind: format!("{}", entry.oid.kind()),
                id: format!("{}", entry.oid),
                name: format!("{}", entry.filename),
            }).collect(),
        }
    )
}

pub(super) fn obj_commit_to_info(git_object: &Object) -> ObjectInfo {
    ObjectInfo::CommitInfo(commitref_id_to_commitinfo(
        git_object.id.to_string(),
        CommitRef::from_bytes(&git_object.data)
            .expect("should have been verified as a well-formed commit"),
    ))
}

// practically duplicating the above.
pub(super) fn commit_to_info(commit: &Commit) -> ObjectInfo {
    ObjectInfo::CommitInfo(commitref_id_to_commitinfo(
        commit.id.to_string(),
        CommitRef::from_bytes(&commit.data)
            .expect("should have been verified as a well-formed commit"),
    ))
}

pub(super) fn commitref_id_to_commitinfo(
    commit_id: String,
    commit: CommitRef,
) -> CommitInfo {
    CommitInfo {
        commit_id: commit_id,
        author: format_signature_ref(&commit.author()),
        committer: format_signature_ref(&commit.committer()),
    }
}

pub(super) fn gitresult_to_info<P: Platform>(
    git_result: &GitHandleResult<P>,
    git_object: &Object,
) -> Option<ObjectInfo> {
    // TODO split off to a formatter version?
    // alternatively, produce some structured data?
    match git_object.kind {
        Kind::Blob => {
            Some(obj_blob_to_info(
                &git_object,
                Some(git_result.path),
            ))
        },
        _ => object_to_info(git_object),
    }
}

pub(super) fn object_to_info(
    git_object: &Object,
) -> Option<ObjectInfo> {
    match git_object.kind {
        Kind::Blob => {
            Some(obj_blob_to_info(
                &git_object,
                None,
            ))
        }
        Kind::Tree => {
            Some(obj_tree_to_info(&git_object))
        }
        Kind::Commit => {
            Some(obj_commit_to_info(&git_object))
        }
        Kind::Tag => {
            None
        }
    }
}

