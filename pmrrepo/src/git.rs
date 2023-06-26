use async_recursion::async_recursion;
use futures::stream::StreamExt;
use futures::stream::futures_unordered::FuturesUnordered;
use std::io::Write;
pub use git2::{Repository, Blob, Commit, Object, ObjectType, Tree};
use std::path::{Path, PathBuf};
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

use crate::error::{
    ContentError,
    ExecutionError,
    PathError,
    PmrRepoError,
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
        git_result: &'a GitResult,
    ) -> WorkspaceGitResult<'a> {
        WorkspaceGitResult(&workspace_record, git_result)
    }
}

fn blob_to_info(blob: &Blob, path: Option<&str>) -> ObjectInfo {
    ObjectInfo::FileInfo(FileInfo {
        size: blob.size() as u64,
        binary: blob.is_binary(),
        mime_type: path
            .and_then(|path| mime_guess::from_path(path).first_raw())
            .unwrap_or("application/octet-stream")
            .to_string(),
    })
}

fn tree_to_info(_repo: &Repository, tree: &Tree) -> ObjectInfo {
    ObjectInfo::TreeInfo(
        TreeInfo {
            filecount: tree.len() as u64,
            entries: tree.iter().map(|entry| TreeEntryInfo {
                filemode: format!("{:06o}", entry.filemode()),
                kind: entry.kind().unwrap().str().to_string(),
                id: format!("{}", entry.id()),
                name: entry.name().unwrap().to_string(),
            }).collect(),
        }
    )
}

fn commit_to_info(commit: &Commit) -> ObjectInfo {
    ObjectInfo::CommitInfo(CommitInfo {
        commit_id: format!("{}", commit.id()),
        author: format!("{}", commit.author()),
        committer: format!("{}", commit.committer()),
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
                author: format!("{}", &git_result.commit.author()),
                committer: format!("{}", &git_result.commit.committer()),
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
                author: format!("{}", &git_result.commit.author()),
                committer: format!("{}", &git_result.commit.committer()),
            },
            path: format!("{}", &git_result.path),
            object: (*git_result).into(),
        }
    }
}

fn gitresult_to_info(git_result: &GitResult, git_object: &Object) -> Option<ObjectInfo> {
    // TODO split off to a formatter version?
    // alternatively, produce some structured data?
    match git_object.kind() {
        Some(ObjectType::Blob) => {
            Some(blob_to_info(
                git_object.as_blob().unwrap(),
                Some(git_result.path),
            ))
        },
        _ => object_to_info(&git_result.repo, git_object),
    }
}

fn object_to_info(repo: &Repository, git_object: &Object) -> Option<ObjectInfo> {
    // TODO split off to a formatter version?
    // alternatively, produce some structured data?
    match git_object.kind() {
        Some(ObjectType::Blob) => {
            Some(blob_to_info(
                git_object.as_blob().unwrap(),
                None,
            ))
        }
        Some(ObjectType::Tree) => {
            Some(tree_to_info(&repo, git_object.as_tree().unwrap()))
        }
        Some(ObjectType::Commit) => {
            Some(commit_to_info(git_object.as_commit().unwrap()))
        }
        Some(ObjectType::Tag) => {
            None
        }
        Some(ObjectType::Any) | None => {
            None
        }
    }
}

impl From<&GitResult<'_>> for Option<ObjectInfo> {
    fn from(git_result: &GitResult) -> Self {
        match &git_result.target {
            GitResultTarget::Object(object) => {
                object_to_info(&git_result.repo, &object)
            }
            _ => None
        }
    }
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

pub async fn stream_blob(mut writer: impl Write, blob: &Blob<'_>) -> std::result::Result<usize, std::io::Error> {
    writer.write(blob.content())
}

fn get_submodule_target<P: PmrBackend>(
    pmrbackend: &PmrBackendWR<P>,
    commit: &Commit,
    path: &str,
) -> Result<String, PmrRepoError> {
    let obj = commit
        .tree()?
        .get_path(Path::new(".gitmodules"))?
        .to_object(&pmrbackend.repo)?;
    let blob = match std::str::from_utf8(
        obj.as_blob()
        .ok_or(PmrRepoError::ContentError(ContentError::Invalid {
            workspace_id: pmrbackend.workspace.id,
            oid: commit.id().to_string(),
            path: path.to_string(),
            msg: format!("expected to be a blob"),
        }))?
        .content()
    ) {
        Ok(blob) => blob,
        Err(e) => return Err(ContentError::Invalid {
            workspace_id: pmrbackend.workspace.id,
            oid: commit.id().to_string(),
            path: path.to_string(),
            msg: format!("error parsing `.gitmodules`: {}", e),
        }.into())
    };
    let config = match gix::config::File::try_from(blob) {
        Ok(config) => config,
        Err(e) => return Err(ContentError::Invalid {
            workspace_id: pmrbackend.workspace.id,
            oid: commit.id().to_string(),
            path: path.to_string(),
            msg: format!("error parsing `.gitmodules`: {}", e),
        }.into())
    };
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
        let repo_dir = self.git_root.join(self.workspace.id.to_string());
        let repo_check = Repository::open_bare(&repo_dir);

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
    ) -> Result<Self, PmrRepoError> {
        let repo_dir = git_root.join(workspace.id.to_string());
        let repo = Repository::open_bare(repo_dir)?;
        Ok(Self {
            backend: &backend,
            git_root: git_root,
            workspace: &workspace,
            repo: repo,
        })
    }

    pub async fn index_tags(&self) -> Result<(), PmrRepoError> {
        let backend = self.backend;
        let workspace = &self.workspace;

        // collect all the tags for processing later
        let mut tags = Vec::new();
        self.repo.tag_foreach(|oid, name| {
            match String::from_utf8(name.into()) {
                // swapped position for next part.
                Ok(tag_name) => tags.push((tag_name, format!("{}", oid))),
                // simply omit tags not encoded for utf8
                Err(_) => warn!("a tag for commit_id {} omitted due to invalid utf8 encoding", oid),
            }
            true
        })?;

        tags.iter().map(|(name, oid)| async move {
            match WorkspaceTagBackend::index_workspace_tag(backend, workspace.id, &name, &oid).await {
                Ok(_) => info!("indexed tag: {}", name),
                Err(e) => warn!("tagging error: {:?}", e),
            }
        }).collect::<FuturesUnordered<_>>().collect::<Vec<_>>().await;

        Ok(())
    }

    pub async fn get_obj_by_spec(&self, spec: &str) -> Result<(), PmrRepoError> {
        let obj = self.repo.revparse_single(spec)?;
        info!("Found object {} {}", obj.kind().unwrap().str(), obj.id());
        info!("{:?}", object_to_info(&self.repo, &obj));
        Ok(())
    }

    fn get_commit(
        &'a self,
        commit_id: Option<&'a str>,
    ) -> Result<Commit<'a>, PmrRepoError> {
        // TODO the model should have a field for the default target.
        // TODO the default value should be the default (main?) branch.
        let obj = self.repo.revparse_single(commit_id.unwrap_or("HEAD"))?;
        match obj.kind() {
            Some(kind) if kind == ObjectType::Commit => {
                info!("Found {} {}", kind, obj.id());
            }
            Some(_) | None => return Err(PathError::NoSuchCommit {
                workspace_id: self.workspace.id,
                oid: commit_id.unwrap_or("HEAD?").into(),
            }.into())
        }
        match obj.into_commit() {
            Ok(commit) => Ok(commit),
            Err(obj) => Err(ExecutionError::Unexpected {
                workspace_id: self.workspace.id,
                msg: format!("libgit2 said oid {:?} was a commit?", obj.id()),
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
        let tree = commit.tree()?;
        let location: String;
        let location_commit: String;
        info!("Found tree {}", tree.id());

        let (path, target) = match path {
            Some("") | Some("/") | None => {
                info!("No path provided; using root tree entry");
                ("".as_ref(), GitResultTarget::Object(tree.into_object()))
            },
            Some(s) => {
                let path = s.strip_prefix('/').unwrap_or(&s);

                let mut curr_tree = tree;
                let mut curr_path = PathBuf::new();
                let mut target: Option<GitResultTarget> = None;

                let mut comps = Path::new(path).components();

                while let Some(component) = comps.next() {
                    let entry = curr_tree.get_path(Path::new(&component))?;
                    curr_path.push(component);
                    match entry.kind() {
                        Some(ObjectType::Tree) => {
                            curr_tree = entry
                                .to_object(&self.repo)?
                                .into_tree()
                                .unwrap();
                            info!("got {:?}", curr_tree);
                        },
                        Some(ObjectType::Commit) => {
                            info!("entry at {:?} a commit", entry.id());
                            location = get_submodule_target(
                                &self,
                                &commit,
                                curr_path.to_str().unwrap(),
                            )?;
                            location_commit = entry.id().to_string();
                            target = Some(GitResultTarget::SubRepoPath {
                                location: location,
                                commit: location_commit,
                                path: comps.as_path().to_str().unwrap(),
                            });
                            break;
                        }
                        _ => {
                            info!("path {:?} not a tree", &curr_path);
                        }
                    }
                    target = match entry.to_object(&self.repo) {
                        Ok(git_object) => Some(GitResultTarget::Object(git_object)),
                        Err(e) => {
                            info!("Will error later due to {}", e);
                            None
                        }
                    }
                };
                (path, target.unwrap())
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
            GitResultTarget::Object(object) => match object.kind() {
                Some(ObjectType::Blob) => {
                    match object.as_blob() {
                        Some(blob) => Ok(stream_blob(writer, blob).await?),
                        None => Err(ContentError::Invalid {
                            workspace_id: self.workspace.id,
                            oid: git_result.commit.id().to_string(),
                            path: git_result.path.to_string(),
                            msg: format!("expected to be a blob"),
                        }.into())
                    }
                }
                Some(_) | None => Err(ContentError::Invalid {
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
        let mut revwalk = self.repo.revwalk()?;
        revwalk.set_sorting(git2::Sort::TIME)?;
        revwalk.push(commit.id())?;

        let log_entries = revwalk
            .filter_map(|id| {
                let id = match id {
                    Ok(t) => t,
                    // Err(e) => return Some(Err(e)),
                    Err(_) => return None,
                };
                let commit = match self.repo.find_commit(id) {
                    Ok(t) => t,
                    // Err(e) => return Some(Err(e)),
                    Err(_) => return None,
                };
                // Some(Ok(LogEntryInfo {
                Some(LogEntryInfo {
                    commit_id: format!("{}", commit.id()),
                    author: format!("{}", commit.author()),
                    committer: format!("{}", commit.committer()),
                    commit_timestamp: commit.time().seconds(),
                })
                // }))
            })
            .collect::<Vec<_>>();

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
    use textwrap_macros::dedent;

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
        let repo = Repository::open_bare(td).unwrap();
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

    fn create_repodata() -> (
        TempDir,
        (gix::Repository, Vec<gix::ObjectId>),
        (gix::Repository, Vec<gix::ObjectId>),
        (gix::Repository, Vec<gix::ObjectId>),
    ) {
        use crate::test::GitObj::{
            Blob,
            Commit,
            Tree,
        };

        let tempdir = TempDir::new().unwrap();
        // import1
        let (_, import1) = crate::test::repo_init(
            None, Some(&tempdir.path().join("1")), Some(1111010101)).unwrap();
        let mut import1_oids = <Vec<gix::ObjectId>>::new();
        let mut import2_oids = <Vec<gix::ObjectId>>::new();
        let mut repodata_oids = <Vec<gix::ObjectId>>::new();

        import1_oids.push(crate::test::append_commit_from_objects(
            &import1, Some(1111010110), Some("readme for import1"), vec![
            Blob("README", dedent!("
            this is import1
            ")),
        ]).unwrap());
        import1_oids.push(crate::test::append_commit_from_objects(
            &import1, Some(1111010111), Some("adding import1"), vec![
            Blob("if1", dedent!("
            if1
            ")),
            Blob("README", dedent!("
            The readme for import1.
            ")),
        ]).unwrap());

        // import2
        let (_, import2) = crate::test::repo_init(
            None, Some(&tempdir.path().join("2")), Some(1111020202)).unwrap();
        import2_oids.push(crate::test::append_commit_from_objects(
            &import2, Some(1222020220), Some("readme for import2"), vec![
            Blob("README", dedent!("
            this is import2
            ")),
        ]).unwrap());
        import2_oids.push(crate::test::append_commit_from_objects(
            &import2, Some(1222020221), Some("adding import2"), vec![
            Blob("if2", dedent!("
            if2
            ")),
            Blob("README", dedent!("
            The readme for import2.
            ")),
        ]).unwrap());
        import2_oids.push(crate::test::append_commit_from_objects(
            &import2, Some(1222020222), Some("adding import1 as an import"), vec![
            Commit("import1", &format!("{}", import1_oids[1])),
            Blob(".gitmodules", dedent!(r#"
            [submodule "ext/import1"]
                   path = import1
                   url = http://models.example.com/w/import1
            "#)),
        ]).unwrap());

        // repodata
        let (_, repodata) = crate::test::repo_init(
            None, Some(&tempdir.path().join("3")), Some(1654321000)).unwrap();
        repodata_oids.push(crate::test::append_commit_from_objects(
            &repodata, Some(1666666700), Some("Initial commit of repodata"), vec![
            Blob("file1", dedent!("
            This is file1, initial commit.
            ")),
            Blob("README", dedent!("
            A simple readme file.
            ")),
        ]).unwrap());
        repodata_oids.push(crate::test::append_commit_from_objects(
            &repodata, Some(1666666710), Some("adding import1"), vec![
            Blob(".gitmodules", dedent!(r#"
            [submodule "ext/import1"]
                   path = ext/import1
                   url = http://models.example.com/w/import1
            "#)),
            Tree("ext", vec![
                Commit("import1", &format!("{}", import1_oids[0])),
            ]),
        ]).unwrap());
        repodata_oids.push(crate::test::append_commit_from_objects(
            &repodata, Some(1666666720), Some("adding some files"), vec![
            Tree("dir1", vec![
                Blob("file1", "file1 is new"),
                Blob("file2", "file2 is new"),
                Tree("nested", vec![
                    Blob("file_a", "file_a is new"),
                    Blob("file_b", "file_b is new"),
                ]),
            ]),
        ]).unwrap());
        repodata_oids.push(crate::test::append_commit_from_objects(
            &repodata, Some(1666666730), Some("bumping import1"), vec![
            Tree("ext", vec![
                Commit("import1", &format!("{}", import1_oids[1])),
            ]),
            Blob("file1", dedent!("
            This is file1, initial commit.
            This line added with import1 bump.
            ")),
            Blob("file2", dedent!("
            This is file2, added with import1 bump.
            ")),
        ]).unwrap());
        repodata_oids.push(crate::test::append_commit_from_objects(
            &repodata, Some(1666666740), Some("adding import2"), vec![
            Blob(".gitmodules", dedent!(r#"
            [submodule "ext/import1"]
                   path = ext/import1
                   url = http://models.example.com/w/import1
            [submodule "ext/import2"]
                   path = ext/import2
                   url = http://models.example.com/w/import2
            "#)),
            Tree("ext", vec![
                Commit("import2", &format!("{}", import2_oids[0])),
            ]),
        ]).unwrap());
        repodata_oids.push(crate::test::append_commit_from_objects(
            &repodata, Some(1666666750), Some("bumping import2"), vec![
            Tree("ext", vec![
                Commit("import2", &format!("{}", import2_oids[1])),
            ]),
        ]).unwrap());
        repodata_oids.push(crate::test::append_commit_from_objects(
            &repodata, Some(1666666760),
            Some("bumping import2, breaking import1"), vec![
            Tree("ext", vec![
                Commit("import1", &format!("{}", import2_oids[1])),
                Commit("import2", &format!("{}", import2_oids[2])),
            ]),
        ]).unwrap());
        repodata_oids.push(crate::test::append_commit_from_objects(
            &repodata, Some(1666666770),
            Some("fixing import1"), vec![
            Tree("ext", vec![
                Commit("import1", &format!("{}", import1_oids[1])),
            ]),
        ]).unwrap());
        repodata_oids.push(crate::test::append_commit_from_objects(
            &repodata, Some(1666666780), Some("updating dir1"), vec![
            Tree("dir1", vec![
                Blob("file2", "file2 is modified"),
                Tree("nested", vec![
                    Blob("file_c", "file_c is new"),
                ]),
            ]),
        ]).unwrap());

        (
            tempdir,
            (import1, import1_oids),
            (import2, import2_oids),
            (repodata, repodata_oids),
        )
    }

    #[async_std::test]
    async fn test_workspace_submodule_access() {
        let (
            git_root,
            _, // (import1, import1_oids),
            _, // (import2, import2_oids),
            _, // (repodata, repodata_oids)
        ) = create_repodata();

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
