use async_recursion::async_recursion;
use futures::stream::StreamExt;
use futures::stream::futures_unordered::FuturesUnordered;
use gix::{
    Commit,
    Object,
    Repository,
    object::{
        Kind,
    },
    objs::{
        BlobRef,
        CommitRef,
        TreeRef,
        WriteTo,
        tree::EntryMode,
    },
    traverse::commit::Sorting,
};
use pmrmodel_base::{
    git::{
        TreeEntryInfo,
        LogEntryInfo,
        ObjectInfo,
        FileInfo,
        TreeInfo,
        CommitInfo,
        LogInfo,
        RemoteInfo,

        PathObject,
        PathInfo,
    },
    workspace::{
        WorkspaceRecord,
    },
    workspace_sync::{
        WorkspaceSyncStatus,
    },
    merged::{
        WorkspacePathInfo,
    },
};

use pmrmodel::backend::db::PmrBackend;
use pmrmodel::model::db::workspace::{
    WorkspaceBackend,
};
use pmrmodel::model::db::workspace_sync::{
    WorkspaceSyncBackend,
};
use pmrmodel::model::workspace_sync::{
    fail_sync,
};
use pmrmodel::model::db::workspace_tag::WorkspaceTagBackend;

use std::{
    io::Write,
    ops::Deref,
    path::{
        Path,
        PathBuf,
    },
};

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

pub struct PmrBackendW<'a, P: PmrBackend> {
    backend: &'a P,
    git_root: PathBuf,
    pub workspace: &'a WorkspaceRecord,
}

pub struct PmrBackendWR<'a, P: PmrBackend> {
    backend: &'a P,
    git_root: PathBuf,
    pub workspace: &'a WorkspaceRecord,
    pub repo: Repository,
}

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

pub struct WorkspaceGitResult<'a>(&'a WorkspaceRecord, &'a GitResult<'a>);

impl WorkspaceGitResult<'_> {
    pub fn new<'a>(
        workspace_record: &'a WorkspaceRecord,
        git_result: &'a GitResult<'a>,
    ) -> WorkspaceGitResult<'a> {
        WorkspaceGitResult(&workspace_record, git_result)
    }
}

// These assume the blobs are all contained because the conversion to
// the Ref equivalent currently drops information for gix, and to make
// the internal usage consistent, the raw object is passed.
fn obj_blob_to_info(git_object: &Object, path: Option<&str>) -> ObjectInfo {
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

fn obj_tree_to_info(git_object: &Object) -> ObjectInfo {
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

fn obj_commit_to_info(git_object: &Object) -> ObjectInfo {
    let commit_id = git_object.id.to_string();
    let commit = CommitRef::from_bytes(&git_object.data).unwrap();
    ObjectInfo::CommitInfo(CommitInfo {
        commit_id: commit_id,
        author: format!("{:?}", commit.author()),
        committer: format!("{:?}", commit.committer()),
    })
}

// the annoyance of the three gix types not having a common trait...
// practically duplicating the above.
fn commit_to_info(commit: &Commit) -> ObjectInfo {
    ObjectInfo::CommitInfo(CommitInfo {
        commit_id: commit.id.to_string(),
        author: format!("{:?}", commit.author()),
        committer: format!("{:?}", commit.committer()),
    })
}

impl From<&GitResult<'_>> for Option<PathObject> {
    fn from(git_result: &GitResult) -> Self {
        match &git_result.target {
            GitResultTarget::Object(object) => match gitresult_to_info(
                git_result,
                object,
            ) {
                Some(ObjectInfo::FileInfo(file_info)) => Some(PathObject::FileInfo(file_info)),
                Some(ObjectInfo::TreeInfo(tree_info)) => Some(PathObject::TreeInfo(tree_info)),
                _ => None,
            },
            GitResultTarget::SubRepoPath { location, commit, path } => {
                Some(PathObject::RemoteInfo(RemoteInfo {
                    location: location.to_string(),
                    commit: commit.to_string(),
                    path: path.to_string(),
                }))
            },
        }
    }
}

impl From<&GitResult<'_>> for PathInfo {
    fn from(git_result: &GitResult) -> Self {
        PathInfo {
            commit: CommitInfo {
                commit_id: format!("{}", &git_result.commit.id()),
                author: format!("{:?}", &git_result.commit.author()),
                committer: format!("{:?}", &git_result.commit.committer()),
            },
            path: format!("{}", &git_result.path),
            object: git_result.into(),
        }
    }
}

impl From<&WorkspaceGitResult<'_>> for WorkspacePathInfo {
    fn from(
        WorkspaceGitResult(
            workspace,
            git_result,
        ): &WorkspaceGitResult<'_>
    ) -> Self {
        WorkspacePathInfo {
            workspace_id: workspace.id,
            description: workspace.description.clone(),
            commit: CommitInfo {
                commit_id: format!("{}", &git_result.commit.id()),
                author: format!("{:?}", &git_result.commit.author()),
                committer: format!("{:?}", &git_result.commit.committer()),
            },
            path: format!("{}", &git_result.path),
            object: (*git_result).into(),
        }
    }
}

fn gitresult_to_info(git_result: &GitResult, git_object: &Object) -> Option<ObjectInfo> {
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

fn object_to_info(
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

impl From<&GitResult<'_>> for Option<ObjectInfo> {
    fn from(git_result: &GitResult) -> Self {
        match &git_result.target {
            GitResultTarget::Object(object) => {
                object_to_info(&object)
            }
            _ => None
        }
    }
}

// TODO move to module for gix_support
fn rev_parse_single<'a>(
    repo: &'a Repository,
    commit_id: &'a str,
) -> Result<Object<'a>, GixError> {
    Ok(repo.rev_parse_single(commit_id)?.object()?)
}

pub fn stream_git_result_default(mut writer: impl Write, git_result: &GitResult) -> std::result::Result<usize, std::io::Error> {
    // TODO split off to a formatter version?
    // alternatively, produce some structured data?
    writer.write(format!("
        have repo at {:?}
        have commit {:?}
        have commit_object {:?}
        using repopath {:?}
        have git_object {:?}
        have path_info {:?}
        \n",
        git_result.repo.path(),
        &git_result.commit.id(),
        commit_to_info(&git_result.commit),
        git_result.path,
        <Option<ObjectInfo>>::from(git_result),
        <PathInfo>::from(git_result),
    ).as_bytes())
}

pub fn stream_git_result_as_json(
    writer: impl Write,
    git_result: &GitResult,
) -> Result<(), serde_json::Error> {
    // TODO how to generalize this to deal with a common "theme" of JSON outputs?
    // currently, this is directly coupled to GitResult, but perhaps there needs
    // to be some trait that provide the output desired?
    // Also, need to consider how to provide a more generic JSON-LD builder framework
    // of sort?  Need to build context and what not...
    // generalize a UI based on that schema/grammar?
    serde_json::to_writer(writer, &<PathInfo>::from(git_result))
}

pub async fn stream_blob(
    mut writer: impl Write,
    blob: &Object<'_>,
) -> std::result::Result<usize, std::io::Error> {
    writer.write(&blob.data)
}

fn get_submodule_target<P: PmrBackend>(
    pmrbackend: &PmrBackendWR<P>,
    commit: &Commit,
    path: &str,
) -> Result<String, PmrRepoError> {
    let blob = commit
        .tree_id()
        .map_err(|e| PmrRepoError::from(GixError::from(e)))?
        .object()
        .map_err(|e| PmrRepoError::from(GixError::from(e)))?
        .try_into_tree()
        .map_err(|e| PmrRepoError::from(GixError::from(e)))?
        .lookup_entry_by_path(Path::new(".gitmodules"))
        .map_err(|e| PmrRepoError::from(GixError::from(e)))?
        .ok_or_else(|| PmrRepoError::from(PathError::NoSuchPath {
            workspace_id: pmrbackend.workspace.id,
            oid: commit.id.to_string(),
            path: path.to_string(),
        }))?
        .id()
        .object()
        .map_err(|e| PmrRepoError::from(GixError::from(e)))?;
    let config = gix::config::File::try_from(
        std::str::from_utf8(&blob.data)
        .map_err(
            |e| PmrRepoError::from(ContentError::Invalid {
                workspace_id: pmrbackend.workspace.id,
                oid: commit.id().to_string(),
                path: path.to_string(),
                msg: format!("error parsing `.gitmodules`: {}", e),
            })
        )?
    ).map_err(
        |e| PmrRepoError::from(ContentError::Invalid {
            workspace_id: pmrbackend.workspace.id,
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
        workspace_id: pmrbackend.workspace.id,
        oid: commit.id().to_string(),
        path: path.to_string(),
    }.into())
}

// If trait aliases <https://github.com/rust-lang/rust/issues/41517> are stabilized:
// pub trait PmrWorkspaceBackend = PmrBackend + WorkspaceBackend + WorkspaceSyncBackend + WorkspaceTagBackend;
pub trait PmrWorkspaceBackend: PmrBackend
    + WorkspaceBackend
    + WorkspaceSyncBackend
    + WorkspaceTagBackend {}
impl<P: PmrBackend
    + WorkspaceBackend
    + WorkspaceSyncBackend
    + WorkspaceTagBackend> PmrWorkspaceBackend for P {}

impl<'a, P: PmrWorkspaceBackend> PmrBackendW<'a, P> {
    pub fn new(
        backend: &'a P,
        git_root: PathBuf,
        workspace: &'a WorkspaceRecord,
    ) -> Self {
        Self {
            backend: &backend,
            git_root: git_root,
            workspace: &workspace,
        }
    }

    pub async fn git_sync_workspace(self) -> Result<PmrBackendWR<'a, P>, PmrRepoError> {
        // using libgit2 as mature protocol support is desired.
        let repo_dir = self.git_root.join(self.workspace.id.to_string());
        let repo_check = git2::Repository::open_bare(&repo_dir);

        info!("Syncing local {:?} with remote <{}>...", repo_dir, &self.workspace.url);
        let sync_id = WorkspaceSyncBackend::begin_sync(self.backend, self.workspace.id).await?;
        match repo_check {
            Ok(repo) => {
                info!("Found existing repo at {:?}, synchronizing...", repo_dir);
                let mut remote = repo.find_remote("origin")?;
                match remote.fetch(&[] as &[&str], None, None) {
                    Ok(_) => info!("Repository synchronized"),
                    Err(e) => {
                        fail_sync(self.backend, sync_id).await?;
                        return Err(ExecutionError::Synchronize {
                            workspace_id: self.workspace.id,
                            remote: self.workspace.url.clone(),
                            msg: e.to_string(),
                        }.into())
                    },
                };
            }
            Err(ref e) if e.class() == git2::ErrorClass::Repository => {
                fail_sync(self.backend, sync_id).await?;
                return Err(ExecutionError::Synchronize {
                    workspace_id: self.workspace.id,
                    remote: self.workspace.url.clone(),
                    msg: "expected local underlying repo to be a bare repo".to_string(),
                }.into())
            }
            Err(_) => {
                info!("Cloning new repository at {:?}...", repo_dir);
                let mut builder = git2::build::RepoBuilder::new();
                builder.bare(true);
                match builder.clone(&self.workspace.url, &repo_dir) {
                    Ok(_) => info!("Repository cloned"),
                    Err(e) => {
                        fail_sync(self.backend, sync_id).await?;
                        return Err(ExecutionError::Synchronize {
                            workspace_id: self.workspace.id,
                            remote: self.workspace.url.clone(),
                            msg: format!("fail to clone: {}", e),
                        }.into())
                    },
                };
            }
        }

        WorkspaceSyncBackend::complete_sync(self.backend, sync_id, WorkspaceSyncStatus::Completed).await?;
        let result = PmrBackendWR::new(self.backend, self.git_root, &self.workspace)?;
        result.index_tags().await?;

        Ok(result)
    }
}


impl<'a, P: PmrWorkspaceBackend> PmrBackendWR<'a, P> {
    pub fn new(
        backend: &'a P,
        git_root: PathBuf,
        workspace: &'a WorkspaceRecord,
    ) -> Result<Self, GixError> {
        let repo_dir = git_root.join(workspace.id.to_string());
        let repo = gix::open::Options::isolated()
            .open_path_as_is(true)
            .open(repo_dir)?
            .to_thread_local();
        Ok(Self {
            backend: &backend,
            git_root: git_root,
            workspace: &workspace,
            repo: repo,
        })
    }

    pub async fn index_tags(&self) -> Result<(), GixError> {
        let backend = self.backend;
        let workspace = &self.workspace;
        self.repo.references()?.tags()?.map(|reference| {
            match reference {
                Ok(tag) => {
                    let target = tag.target().id().to_hex().to_string();
                    match std::str::from_utf8(tag.name().as_bstr().deref()) {
                        Ok(s) => Some((s.to_string(), target)),
                        Err(_) => {
                            warn!("\
                            a tag for commit_id {} omitted due to \
                            invalid utf8 encoding\
                            ", target
                            );
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!("failed to decode a tag: {}", e);
                    None
                }
            }
        }).filter_map(|x| x)
            .map(|(name, oid)| async move {
                match WorkspaceTagBackend::index_workspace_tag(
                    backend,
                    workspace.id,
                    &name,
                    &oid,
                ).await {
                    Ok(_) => info!("indexed tag: {}", name),
                    Err(e) => warn!("tagging error: {:?}", e),
                }
            })
            .collect::<FuturesUnordered<_>>().collect::<Vec<_>>().await;

        Ok(())
    }

    pub async fn get_obj_by_spec(&self, spec: &str) -> Result<(), GixError> {
        let obj = self.repo.rev_parse_single(spec)?.object()?;
        info!("Found object {} {}", obj.kind, obj.id);
        info!("{:?}", object_to_info(&obj));
        Ok(())
    }

    fn get_commit(
        &'a self,
        commit_id: Option<&'a str>,
    ) -> Result<Commit<'a>, PmrRepoError> {
        let obj = rev_parse_single(&self.repo, &commit_id.unwrap_or("HEAD"))?;
        match obj.kind {
            kind if kind == Kind::Commit => {
                info!("Found {} {}", kind, obj.id);
            }
            _ => return Err(PathError::NoSuchCommit {
                workspace_id: self.workspace.id,
                oid: commit_id.unwrap_or("HEAD?").into(),
            }.into())
        }
        match obj.try_into_commit() {
            Ok(commit) => Ok(commit),
            Err(obj) => Err(ExecutionError::Unexpected {
                workspace_id: self.workspace.id,
                msg: format!("libgit2 said oid {:?} was a commit?", obj.id),
            }.into()),
        }
    }

    // commit_id/path should be a pathinfo struct?
    pub fn pathinfo(
        &self,
        commit_id: Option<&'a str>,
        path: Option<&'a str>,
    ) -> Result<GitResult, PmrRepoError> {
        let commit = self.get_commit(commit_id)?;
        let tree = commit.tree_id()
            .map_err(|e| PmrRepoError::from(GixError::from(e)))?
            .object()
            .map_err(|e| PmrRepoError::from(GixError::from(e)))?;

        let (path, target) = match path {
            Some("") | Some("/") | None => {
                info!("No path provided; using root tree entry");
                ("".as_ref(), GitResultTarget::Object(tree))
            },
            Some(s) => {
                let path = s.strip_prefix('/').unwrap_or(&s);
                let mut comps = Path::new(path).components();
                let mut curr_path = PathBuf::new();
                let mut object = Some(tree);
                let mut target: Option<GitResultTarget> = None;

                while let Some(component) = comps.next() {
                    let entry = object
                        .expect("iteration has this set or look breaked")
                        .try_into_tree()
                        .map_err(|e| PmrRepoError::from(GixError::from(e)))?
                        .lookup_entry_by_path(Path::new(&component))
                        .map_err(|e| PmrRepoError::from(GixError::from(e)))?
                        .ok_or_else(
                            || PmrRepoError::from(PathError::NoSuchPath {
                                workspace_id: self.workspace.id,
                                oid: commit.id.to_string(),
                                path: path.to_string(),
                            })
                        )?;
                    curr_path.push(component);
                    match entry.mode() {
                        EntryMode::Commit => {
                            info!("entry {:?} is a commit", entry.id());
                            let location = get_submodule_target(
                                &self,
                                &commit,
                                curr_path.to_str().unwrap(),
                            )?;
                            target = Some(GitResultTarget::SubRepoPath {
                                location: location,
                                commit: entry.id().to_string(),
                                path: comps.as_path().to_str().unwrap(),
                            });
                            object = None;
                            break;
                        }
                        _ => ()
                    }
                    let next_object = entry
                        .object()
                        .map_err(|e| PmrRepoError::from(GixError::from(e)))?;
                    info!("got {} {:?}", next_object.kind, &next_object);
                    object = Some(next_object);
                };
                match object {
                    Some(object) =>
                        (path, GitResultTarget::Object(object)),
                    None =>
                        // Only way object is None is have target set.
                        (path, target.expect("to be a SubRepoPath")),
                }
            },
        };
        let git_result = GitResult {
            repo: &self.repo,
            commit: commit,
            path: path,
            target: target,
        };
        Ok(git_result)
    }

    #[async_recursion(?Send)]
    pub async fn stream_result_blob(
        &self,
        writer: impl Write + 'async_recursion,
        git_result: &GitResult<'a>,
    ) -> Result<usize, PmrRepoError> {
        match &git_result.target {
            GitResultTarget::Object(object) => match object.kind {
                Kind::Blob => Ok(stream_blob(writer, object).await?),
                _ => Err(ContentError::Invalid {
                    workspace_id: self.workspace.id,
                    oid: git_result.commit.id().to_string(),
                    path: git_result.path.to_string(),
                    msg: format!("expected to be a blob"),
                }.into())
            },
            GitResultTarget::SubRepoPath { location, commit, path } => {
                let workspace = WorkspaceBackend::get_workspace_by_url(
                    self.backend, &location,
                ).await?;
                let pmrbackend = PmrBackendWR::new(
                    self.backend, self.git_root.clone(), &workspace)?;
                let git_result = pmrbackend.pathinfo(
                    Some(commit), Some(path),
                )?;
                Ok(self.stream_result_blob(writer, &git_result).await?)
            },
        }
    }

    pub fn loginfo(
        &self,
        commit_id: Option<&str>,
        _path: Option<&'a str>,
    ) -> Result<ObjectInfo, PmrRepoError> {
        let commit = self.get_commit(commit_id)?;
        let log_entries = self.repo
            .rev_walk([commit.id])
            .sorting(Sorting::ByCommitTimeNewestFirst)
            .all()
            .map_err(|e| PmrRepoError::from(GixError::from(e)))?
            .map(|info| {
                let commit = info?.object()?;
                Ok(LogEntryInfo {
                    commit_id: format!("{}", commit.id()),
                    author: format!("{:?}", commit.author()?),
                    committer: format!("{:?}", commit.committer()?),
                    // We are not going to bother with commit timestamps
                    // that go beyond i64; while casting like this will
                    // result in silently breaking stuff, revisit this
                    // bit later when there is more finality in what gix
                    // does.
                    commit_timestamp: commit.time()?.seconds as i64,
                })
            })
            .collect::<Result<Vec<_>, GixError>>()?;

        let result = ObjectInfo::LogInfo(LogInfo { entries: log_entries });
        Ok(result)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    use async_trait::async_trait;
    use mockall::mock;
    use mockall::predicate::*;
    use tempfile::TempDir;

    // use pmrmodel::backend::db::MockHasPool;
    use pmrmodel_base::workspace_tag::WorkspaceTagRecord;
    use pmrmodel_base::workspace_sync::WorkspaceSyncRecord;
    use pmrmodel::backend::db::PmrBackend;

    mock! {
        Backend {}
        impl PmrBackend for Backend {}

        #[async_trait]
        impl WorkspaceTagBackend for Backend {
            async fn index_workspace_tag(&self, workspace_id: i64, name: &str, commit_id: &str) -> Result<i64, sqlx::Error>;
            async fn get_workspace_tags(&self, workspace_id: i64) -> Result<Vec<WorkspaceTagRecord>, sqlx::Error>;
        }

        #[async_trait]
        impl WorkspaceBackend for Backend {
            async fn add_workspace(
                &self, url: &str, description: &str, long_description: &str
            ) -> Result<i64, sqlx::Error>;
            async fn update_workspace(
                &self, id: i64, description: &str, long_description: &str
            ) -> Result<bool, sqlx::Error>;
            async fn list_workspaces(&self) -> Result<Vec<WorkspaceRecord>, sqlx::Error>;
            async fn get_workspace_by_id(&self, id: i64) -> Result<WorkspaceRecord, sqlx::Error>;
            async fn get_workspace_by_url(&self, url: &str) -> Result<WorkspaceRecord, sqlx::Error>;
        }

        #[async_trait]
        impl WorkspaceSyncBackend for Backend {
            async fn begin_sync(&self, workspace_id: i64) -> Result<i64, sqlx::Error>;
            async fn complete_sync(&self, id: i64, status: WorkspaceSyncStatus) -> Result<bool, sqlx::Error>;
            async fn get_workspaces_sync_records(&self, workspace_id: i64) -> Result<Vec<WorkspaceSyncRecord>, sqlx::Error>;
        }
    }

    // helper to deal with moves of the workspace record.
    async fn git_sync_helper(
        mock_backend: &MockBackend, id: i64, url: &str, git_root: &TempDir
    ) -> Result<(), PmrRepoError> {
        let workspace = WorkspaceRecord { id: id, url: url.to_string(), description: None };
        let pmrbackend = PmrBackendW::new(mock_backend, git_root.path().to_owned().to_path_buf(), &workspace);
        pmrbackend.git_sync_workspace().await?;
        Ok(())
    }

    #[async_std::test]
    async fn test_git_sync_workspace_empty() {
        let (td_, _) = crate::test::repo_init(None, None, None).unwrap();
        let td = td_.unwrap();
        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(0))
            .returning(|_| Ok(1));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(1), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));
        let git_root = TempDir::new().unwrap();
        git_sync_helper(&mock_backend, 0, td.path().to_str().unwrap(), &git_root).await.unwrap();
    }

    #[async_std::test]
    async fn test_git_sync_workspace_with_index_tag() {
        let (td_, _) = crate::test::repo_init(None, None, None).unwrap();
        let td = td_.as_ref().unwrap();
        // TODO use gix to tag?
        let repo = git2::Repository::open_bare(td).unwrap();
        let id = repo.head().unwrap().target().unwrap();
        let obj = repo.find_object(id, None).unwrap();
        repo.tag_lightweight("new_tag", &obj, false).unwrap();

        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(123))
            .returning(|_| Ok(1));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(1), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));
        mock_backend.expect_index_workspace_tag()
            .times(1)
            .withf(move |workspace_id: &i64, name: &str, commit_id: &str| {
                *workspace_id == 123 && name == "refs/tags/new_tag" && commit_id == format!("{}", id)
            })
            .returning(|_, _, _| Ok(1));

        let git_root = TempDir::new().unwrap();
        git_sync_helper(&mock_backend, 123, td.path().to_str().unwrap(), &git_root).await.unwrap();
    }

    #[async_std::test]
    async fn test_git_sync_failure_invalid_remote() {
        // where remote couldn't be found or invalid.
        let td = TempDir::new().unwrap();
        let err_msg = format!(
            "ExecutionError: workspace `{0}`: failed to synchronize with remote `{1}`: \
            fail to clone: could not find repository from '{1}'; \
            class=Repository (6); code=NotFound (-3)", 2, td.path().to_str().unwrap());
        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(2))
            .returning(|_| Ok(3));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(3), eq(WorkspaceSyncStatus::Error))
            .returning(|_, _| Ok(true));

        let git_root = TempDir::new().unwrap();
        let err = git_sync_helper(
            &mock_backend, 2, td.path().to_str().unwrap(), &git_root).await.unwrap_err();
        assert_eq!(err.to_string(), err_msg);
    }

    #[async_std::test]
    async fn test_git_sync_failure_dropped_source() {
        let (td_, _) = crate::test::repo_init(None, None, None).unwrap();
        let td = td_.unwrap();
        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(42))
            .returning(|_| Ok(1));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(1), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));

        let git_root = TempDir::new().unwrap();

        let td_path = td.path().to_owned();
        let url = td_path.to_str().unwrap();
        assert!(git_sync_helper(&mock_backend, 42, url, &git_root).await.is_ok());

        td.close().unwrap();
        mock_backend.checkpoint();

        // now verify that the failure to sync will generate the right error
        // when an originally working remote disappeared or errored.
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(42))
            .returning(|_| Ok(2));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(2), eq(WorkspaceSyncStatus::Error))
            .returning(|_, _| Ok(true));

        let failed_sync = git_sync_helper(&mock_backend, 42, url, &git_root).await;
        let err_msg = format!(
            "ExecutionError: workspace `42`: failed to synchronize with \
            remote `{}`: unsupported URL protocol; class=Net (12)", url
        );
        assert_eq!(failed_sync.unwrap_err().to_string(), err_msg);
    }

    #[async_std::test]
    async fn test_git_sync_workspace_not_bare() {
        let (origin_, _) = crate::test::repo_init(None, None, None).unwrap();
        let origin = origin_.unwrap();

        let git_root_dir = TempDir::new().unwrap();
        let repo_dir = git_root_dir.path().join("10");

        let mut repo = gix::ThreadSafeRepository::init_opts(
            &repo_dir,
            gix::create::Kind::WithWorktree,
            gix::create::Options::default(),
            gix::open::Options::isolated(),
        ).unwrap().to_thread_local();
        let mut config = repo.config_snapshot_mut();
        config.set_raw_value("committer", None, "name", "user").unwrap();
        config.set_raw_value("committer", None, "email", "user@example.com").unwrap();
        drop(config);
        crate::test::init_empty_commit(&repo, None).unwrap();
        crate::test::commit(&repo, vec![("some_file", "")]).unwrap();

        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(10))
            .returning(|_| Ok(1));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(1), eq(WorkspaceSyncStatus::Error))
            .returning(|_, _| Ok(true));
        let failed_sync = git_sync_helper(
            &mock_backend, 10, origin.path().to_str().unwrap(), &git_root_dir
        ).await.unwrap_err();
        let err_msg = format!(
            "ExecutionError: workspace `10`: failed to synchronize with \
            remote `{}`: expected local underlying repo to be a bare repo",
            origin.path().display(),
        );
        assert_eq!(failed_sync.to_string(), err_msg);
    }

    #[async_std::test]
    async fn test_workspace_path_info_from_workspace_git_result() {
        let (td_, repo) = crate::test::repo_init(None, None, None).unwrap();
        crate::test::commit(&repo, vec![("some_file", "")]).unwrap();

        let td = td_.unwrap();
        let td_path = td.path().to_owned();
        let url = td_path.to_str().unwrap();

        let git_root = TempDir::new().unwrap();
        let mut mock_backend = MockBackend::new();
        mock_backend.expect_begin_sync()
            .times(1)
            .with(eq(10))
            .returning(|_| Ok(10));
        mock_backend.expect_complete_sync()
            .times(1)
            .with(eq(10), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));
        assert!(git_sync_helper(&mock_backend, 10, url, &git_root).await.is_ok());

        let workspace = WorkspaceRecord {
            id: 10,
            url: "http://example.com/10".to_string(),
            description: Some("demo workspace 10".to_string())
        };

        let pmrbackend = PmrBackendWR::new(
            &mock_backend,
            git_root.path().to_path_buf(),
            &workspace,
        ).unwrap();

        let result = pmrbackend.pathinfo(None, None).unwrap();
        let pathinfo = <WorkspacePathInfo>::from(&WorkspaceGitResult::new(&pmrbackend.workspace, &result));
        assert_eq!(pathinfo.path, "".to_string());
        assert_eq!(pathinfo.description, Some("demo workspace 10".to_string()));
    }

    #[async_std::test]
    async fn test_workspace_submodule_access() {
        let (
            git_root,
            _, // (import1, import1_oids),
            _, // (import2, import2_oids),
            _, // (repodata, repodata_oids)
        ) = crate::test::create_repodata();

        // let import1_workspace = WorkspaceRecord {
        //     id: 1,
        //     url: "http://models.example.com/w/import1".to_string(),
        //     description: Some("The import1 workspace".to_string())
        // };
        // let import2_workspace = WorkspaceRecord {
        //     id: 2,
        //     url: "http://models.example.com/w/import2".to_string(),
        //     description: Some("The import2 workspace".to_string())
        // };
        let repodata_workspace = WorkspaceRecord {
            id: 3,
            url: "http://models.example.com/w/repodata".to_string(),
            description: Some("The repodata workspace".to_string())
        };

        let mut mock_backend = MockBackend::new();
        // used later.
        mock_backend.expect_get_workspace_by_url()
            .times(1)
            .with(eq("http://models.example.com/w/import2"))
            .returning(|_| Ok(WorkspaceRecord {
                id: 2,
                url: "http://models.example.com/w/import2".to_string(),
                description: Some("The import2 workspace".to_string())
            }));
        let pmrbackend = PmrBackendWR::new(
            &mock_backend,
            git_root.path().to_path_buf(),
            &repodata_workspace,
        ).unwrap();

        let result = pmrbackend.pathinfo(
            Some("557ee3cb13fb421d2bd6897615ae95830eb427c8"),
            Some("ext/import1/README"),
        ).unwrap();

        let pathinfo = <WorkspacePathInfo>::from(&WorkspaceGitResult::new(
            &pmrbackend.workspace, &result));
        assert_eq!(
            pathinfo.path,
            "ext/import1/README".to_string());
        assert_eq!(
            pathinfo.object,
            Some(PathObject::RemoteInfo(RemoteInfo {
                location: "http://models.example.com/w/import1"
                    .to_string(),
                commit: "01b952d14a0a33d22a0aa465fe763e5d17b15d46"
                    .to_string(),
                path: "README".to_string(),
            })),
        );

        let result = pmrbackend.pathinfo(
            Some("c4d735e5a305559c1cb0ce8de4c25ed5c3f4f263"),
            Some("ext/import2/import1/if1"),
        ).unwrap();
        let pathinfo = <WorkspacePathInfo>::from(&WorkspaceGitResult::new(
            &pmrbackend.workspace, &result));
        assert_eq!(
            pathinfo.path,
            "ext/import2/import1/if1".to_string());
        assert_eq!(
            pathinfo.object,
            Some(PathObject::RemoteInfo(RemoteInfo {
                location: "http://models.example.com/w/import2"
                    .to_string(),
                commit: "0ab8a26a0e85a033bea0388216667d83cc0dc1dd"
                    .to_string(),
                path: "import1/if1".to_string(),
            })),
        );

        let mut buffer = <Vec<u8>>::new();
        let readme_result = pmrbackend.pathinfo(
            Some("557ee3cb13fb421d2bd6897615ae95830eb427c8"),
            Some("README"),
        ).unwrap();
        assert_eq!(
            pmrbackend.stream_result_blob(&mut buffer, &readme_result).await.unwrap(),
            22,
        );
        assert_eq!(
            std::str::from_utf8(&buffer).unwrap(),
            "A simple readme file.\n",
        );

        let mut buffer = <Vec<u8>>::new();
        let import2_result = pmrbackend.pathinfo(
            Some("a4a04eed5e243e3019592579a7f6eb950399f9bf"),
            Some("ext/import2/if2"),
        ).unwrap();
        assert_eq!(
            pmrbackend.stream_result_blob(&mut buffer, &import2_result).await.unwrap(),
            4,
        );
        assert_eq!(
            std::str::from_utf8(&buffer).unwrap(),
            "if2\n",
        );

    }


}
