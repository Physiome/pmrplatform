use anyhow::bail;
use futures::stream::StreamExt;
use futures::stream::futures_unordered::FuturesUnordered;
use std::io::Write;
use git2::{Repository, Blob, Commit, Object, ObjectType, Tree};
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
    merged::{
        WorkspacePathInfo,
    },
};

use crate::backend::db::PmrBackend;
use crate::model::workspace::{
    WorkspaceBackend,
};
use crate::model::workspace_sync::{
    WorkspaceSyncBackend,
    WorkspaceSyncStatus,
    fail_sync,
};
use crate::model::workspace_tag::WorkspaceTagBackend;

pub struct GitPmrAccessor<'a, P: PmrBackend> {
    backend: &'a P,
    git_root: PathBuf,
    pub workspace: WorkspaceRecord,
}

pub enum GitResultTarget<'a> {
    Object(Object<'a>),
    SubRepoPath {
        location: &'a str,
        commit: &'a str,
        path: &'a str,
    },
}

pub struct GitResultSet<'a> {
    pub repo: &'a Repository,
    pub commit: &'a Commit<'a>,
    pub path: &'a str,
    pub target: GitResultTarget<'a>,
}

pub struct WorkspaceGitResultSet<'a>(&'a WorkspaceRecord, &'a GitResultSet<'a>);

impl WorkspaceGitResultSet<'_> {
    pub fn new<'a>(
        workspace_record: &'a WorkspaceRecord,
        git_result_set: &'a GitResultSet,
    ) -> WorkspaceGitResultSet<'a> {
        WorkspaceGitResultSet(&workspace_record, git_result_set)
    }
}

fn blob_to_info(blob: &Blob) -> ObjectInfo {
    ObjectInfo::FileInfo(FileInfo {
        size: blob.size() as u64,
        binary: blob.is_binary(),
    })
}

fn tree_to_info(repo: &Repository, tree: &Tree) -> ObjectInfo {
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

fn gitresultset_target_to_pathobject(
    git_result_set: &GitResultSet,
) -> Option<PathObject> {
    // TODO None may represent error here?
    match &git_result_set.target {
        GitResultTarget::Object(object) => match object_to_info(
            &git_result_set.repo,
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

impl From<&GitResultSet<'_>> for PathInfo {
    fn from(git_result_set: &GitResultSet) -> Self {
        PathInfo {
            commit: CommitInfo {
                commit_id: format!("{}", &git_result_set.commit.id()),
                author: format!("{}", &git_result_set.commit.author()),
                committer: format!("{}", &git_result_set.commit.committer()),
            },
            path: format!("{}", &git_result_set.path),
            object: gitresultset_target_to_pathobject(git_result_set),
        }
    }
}

impl From<&WorkspaceGitResultSet<'_>> for WorkspacePathInfo {
    fn from(
        WorkspaceGitResultSet(
            workspace,
            git_result_set,
        ): &WorkspaceGitResultSet<'_>
    ) -> Self {
        WorkspacePathInfo {
            workspace_id: workspace.id,
            description: workspace.description.clone(),
            commit: CommitInfo {
                commit_id: format!("{}", &git_result_set.commit.id()),
                author: format!("{}", &git_result_set.commit.author()),
                committer: format!("{}", &git_result_set.commit.committer()),
            },
            path: format!("{}", &git_result_set.path),
            object: gitresultset_target_to_pathobject(git_result_set),
        }
    }
}

fn object_to_info(repo: &Repository, git_object: &Object) -> Option<ObjectInfo> {
    // TODO split off to a formatter version?
    // alternatively, produce some structured data?
    match git_object.kind() {
        Some(ObjectType::Blob) => {
            Some(blob_to_info(git_object.as_blob().unwrap()))
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

impl From<&GitResultSet<'_>> for Option<ObjectInfo> {
    fn from(git_result_set: &GitResultSet) -> Self {
        match &git_result_set.target {
            GitResultTarget::Object(object) => {
                object_to_info(&git_result_set.repo, &object)
            }
            _ => None
        }
    }
}

pub fn stream_git_result_set_default(mut writer: impl Write, git_result_set: &GitResultSet) -> std::result::Result<usize, std::io::Error> {
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
        git_result_set.repo.path(),
        &git_result_set.commit.id(),
        commit_to_info(&git_result_set.commit),
        git_result_set.path,
        <Option<ObjectInfo>>::from(git_result_set),
        <PathInfo>::from(git_result_set),
    ).as_bytes())
}

pub fn stream_git_result_set_as_json(
    writer: impl Write,
    git_result_set: &GitResultSet,
) -> Result<(), serde_json::Error> {
    // TODO how to generalize this to deal with a common "theme" of JSON outputs?
    // currently, this is directly coupled to GitResultSet, but perhaps there needs
    // to be some trait that provide the output desired?
    // Also, need to consider how to provide a more generic JSON-LD builder framework
    // of sort?  Need to build context and what not...
    // generalize a UI based on that schema/grammar?
    serde_json::to_writer(writer, &<PathInfo>::from(git_result_set))
}

pub fn stream_blob(mut writer: impl Write, blob: &Blob) -> std::result::Result<usize, std::io::Error> {
    writer.write(blob.content())
}

pub fn stream_git_result_set_as_blob(writer: impl Write, git_result_set: &GitResultSet) -> anyhow::Result<()> {
    match &git_result_set.target {
        GitResultTarget::Object(object) => match object.kind() {
            Some(ObjectType::Blob) => {
                match &object.as_blob() {
                    Some(blob) => {
                        stream_blob(writer, blob)?;
                        Ok(())
                    }
                    None => bail!("failed to get blob from object")
                }
            }
            Some(_) | None => {
                bail!("target is not a git blob")
            }
        }
        _ => bail!("target is not a git blob")
    }
}

fn get_submodule_target(
    repo: &Repository,
    tree: &Tree,
    path: &str,
) -> anyhow::Result<String> {
    let obj = tree.get_path(Path::new(".gitmodules"))?.to_object(&repo)?;
    let blob = std::str::from_utf8(obj.as_blob().unwrap().content())?;
    let config = git_config::File::try_from(blob)?;
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
    bail!("no submodule declared at {}", path)
}

// If trait aliases <https://github.com/rust-lang/rust/issues/41517> are stabilized:
// pub trait PmrWorkspaceBackend = PmrBackend + WorkspaceBackend + WorkspaceSyncBackend + WorkspaceTagBackend;
pub trait PmrWorkspaceBackend: PmrBackend + WorkspaceBackend + WorkspaceSyncBackend + WorkspaceTagBackend {}
impl<P: PmrBackend + WorkspaceBackend + WorkspaceSyncBackend + WorkspaceTagBackend> PmrWorkspaceBackend for P {}

impl<'a, P: PmrWorkspaceBackend> GitPmrAccessor<'a, P> {
    pub fn new(backend: &'a P, git_root: PathBuf, workspace: WorkspaceRecord) -> Self {
        Self {
            backend: &backend,
            git_root: git_root,
            workspace: workspace,
        }
    }

    pub async fn git_sync_workspace(&self) -> anyhow::Result<()> {
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
                    Err(e) => fail_sync(self.backend, sync_id, format!("Failed to synchronize: {}", e)).await?,
                };
            },
            Err(ref e) if e.class() == git2::ErrorClass::Repository => fail_sync(
                self.backend, sync_id, format!(
                    "Invalid data at local {:?} - expected bare repo", repo_dir)).await?,
            Err(_) => {
                info!("Cloning new repository at {:?}...", repo_dir);
                let mut builder = git2::build::RepoBuilder::new();
                builder.bare(true);
                match builder.clone(&self.workspace.url, &repo_dir) {
                    Ok(_) => info!("Repository cloned"),
                    Err(e) => fail_sync(self.backend, sync_id, format!("Failed to clone: {}", e)).await?,
                };
            }
        }

        WorkspaceSyncBackend::complete_sync(self.backend, sync_id, WorkspaceSyncStatus::Completed).await?;
        self.index_tags().await?;

        Ok(())
    }

    pub async fn index_tags(&self) -> anyhow::Result<()> {
        let backend = self.backend;
        let git_root = &self.git_root;
        let workspace = &self.workspace;
        let repo_dir = git_root.join(workspace.id.to_string());
        let repo = Repository::open_bare(repo_dir)?;

        // collect all the tags for processing later
        let mut tags = Vec::new();
        repo.tag_foreach(|oid, name| {
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

    pub async fn get_obj_by_spec(&self, spec: &str) -> anyhow::Result<()> {
        let git_root = &self.git_root;
        let workspace = &self.workspace;
        let repo_dir = git_root.join(workspace.id.to_string());
        let repo = Repository::open_bare(repo_dir)?;
        let obj = repo.revparse_single(spec)?;
        info!("Found object {} {}", obj.kind().unwrap().str(), obj.id());
        info!("{:?}", object_to_info(&repo, &obj));
        Ok(())
    }

    // commit_id/path should be a pathinfo struct?
    pub async fn process_pathinfo<T>(
        &self,
        commit_id: Option<&str>,
        path: Option<&str>,
        processor: fn(&Self, &GitResultSet) -> T
    ) -> anyhow::Result<T> {
        let git_root = &self.git_root;
        let workspace = &self.workspace;
        let repo_dir = git_root.join(workspace.id.to_string());
        let repo = Repository::open_bare(repo_dir)?;
        // TODO the default value should be the default (main?) branch.
        // TODO the sync procedure should fast forward of sort
        // TODO the model should have a field for main branch
        let obj = repo.revparse_single(commit_id.unwrap_or("origin/HEAD"))?;

        // TODO streamline this a bit.
        match obj.kind() {
            Some(ObjectType::Commit) => {
                info!("Found {} {}", obj.kind().unwrap().str(), obj.id());
            }
            Some(_) | None => bail!("'{}' does not refer to a valid commit", commit_id.unwrap_or(""))
        }
        let commit = obj.as_commit().unwrap();
        let tree = commit.tree()?;
        let location: String;
        let location_commit: String;
        info!("Found tree {}", tree.id());
        // TODO only further navigate into tree_entry if path
        // repopath is the sanitized path to the repo
        let (path, target) = match path {
            Some("") | Some("/") | None => {
                info!("No path provided; using root tree entry");
                ("".as_ref(), GitResultTarget::Object(tree.into_object()))
            },
            Some(s) => {
                let path = if s.chars().nth(0) == Some('/') {
                    &s[1..]
                } else {
                    &s
                };

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
                                .to_object(&repo)?
                                .into_tree()
                                .unwrap();
                            info!("got {:?}", curr_tree);
                        },
                        Some(ObjectType::Commit) => {
                            info!("entry at {:?} a commit", entry.id());
                            location = get_submodule_target(
                                &repo,
                                &commit.tree()?,
                                curr_path.to_str().unwrap(),
                            )?;
                            location_commit = entry.id().to_string();
                            target = Some(GitResultTarget::SubRepoPath {
                                location: &location,
                                commit: &location_commit,
                                path: comps.as_path().to_str().unwrap(),
                            });
                            break;
                        }
                        _ => {
                            info!("path {:?} not a tree", &curr_path);
                        }
                    }
                    target = match entry.to_object(&repo) {
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
        // info!("using git_object {} {}", git_object.kind().unwrap().str(), git_object.id());
        let git_result_set = GitResultSet {
            repo: &repo,
            commit: commit,
            path: path,
            target: target,
        };
        Ok(processor(&self, &git_result_set))
    }

    // pub async fn process_loginfo<T>(
    pub async fn process_loginfo(
        &self,
        commit_id: Option<&str>,
    //     processor: fn(&GitResultSet) -> T
    // ) -> anyhow::Result<T> {
    ) -> anyhow::Result<()> {
        let git_root = &self.git_root;
        let workspace = &self.workspace;
        let repo_dir = git_root.join(workspace.id.to_string());
        let repo = Repository::open_bare(repo_dir)?;
        // TODO the default value should be the default (main?) branch.
        // TODO the sync procedure should fast forward of sort
        // TODO the model should have a field for main branch
        let obj = repo.revparse_single(commit_id.unwrap_or("origin/HEAD"))?;

        // TODO streamline this a bit.
        match obj.kind() {
            Some(ObjectType::Commit) => {
                info!("Found {} {}", obj.kind().unwrap().str(), obj.id());
            },
            Some(_) | None => bail!("'{}' does not refer to a valid commit", commit_id.unwrap_or("")),
        }
        let mut revwalk = repo.revwalk()?;
        revwalk.set_sorting(git2::Sort::TIME)?;
        revwalk.push(obj.id())?;

        let log_entries = revwalk
            .filter_map(|id| {
                let id = match id {
                    Ok(t) => t,
                    // Err(e) => return Some(Err(e)),
                    Err(_) => return None,
                };
                let commit = match repo.find_commit(id) {
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

        info!("{:?}", log_entries);

        let result = ObjectInfo::LogInfo(LogInfo { entries: log_entries });

        info!("{}", serde_json::to_string(&result).unwrap());

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    use async_trait::async_trait;
    use mockall::mock;
    use mockall::predicate::*;
    use tempfile::TempDir;

    // use crate::backend::db::MockHasPool;
    use crate::backend::db::PmrBackend;
    use crate::model::workspace_tag::WorkspaceTagRecord;
    use crate::model::workspace_sync::WorkspaceSyncRecord;

    mock! {
        Backend {}
        impl PmrBackend for Backend {}

        #[async_trait]
        impl WorkspaceTagBackend for Backend {
            async fn index_workspace_tag(&self, workspace_id: i64, name: &str, commit_id: &str) -> anyhow::Result<i64>;
            async fn get_workspace_tags(&self, workspace_id: i64) -> anyhow::Result<Vec<WorkspaceTagRecord>>;
        }

        #[async_trait]
        impl WorkspaceBackend for Backend {
            async fn add_workspace(
                &self, url: &str, description: &str, long_description: &str
            ) -> anyhow::Result<i64>;
            async fn update_workspace(
                &self, id: i64, description: &str, long_description: &str
            ) -> anyhow::Result<bool>;
            async fn list_workspaces(&self) -> anyhow::Result<Vec<WorkspaceRecord>>;
            async fn get_workspace_by_id(&self, id: i64) -> anyhow::Result<WorkspaceRecord>;
        }

        #[async_trait]
        impl WorkspaceSyncBackend for Backend {
            async fn begin_sync(&self, workspace_id: i64) -> anyhow::Result<i64>;
            async fn complete_sync(&self, id: i64, status: WorkspaceSyncStatus) -> anyhow::Result<bool>;
            async fn get_workspaces_sync_records(&self, workspace_id: i64) -> anyhow::Result<Vec<WorkspaceSyncRecord>>;
        }
    }

    // helper to deal with moves of the workspace record.
    async fn git_sync_helper(
        mock_backend: &MockBackend, id: i64, url: &str, git_root: &TempDir
    ) -> anyhow::Result<()> {
        let workspace = WorkspaceRecord { id: id, url: url.to_string(), description: None };
        let git_pmr_accessor = GitPmrAccessor::new(mock_backend, git_root.path().to_owned().to_path_buf(), workspace);
        git_pmr_accessor.git_sync_workspace().await
    }

    #[async_std::test]
    async fn test_git_sync_workspace_empty() {
        let (td_, _) = crate::test::repo_init(None, None);
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
        let (td_, repo) = crate::test::repo_init(None, None);
        let td = td_.unwrap();
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
            "Failed to clone: could not find repository from '{}'; \
            class=Repository (6); code=NotFound (-3)", td.path().to_str().unwrap());
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
        let (td_, _) = crate::test::repo_init(None, None);
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
        let err_msg = "Failed to synchronize: unsupported URL protocol; class=Net (12)";
        assert_eq!(failed_sync.unwrap_err().to_string(), err_msg);
    }

    #[async_std::test]
    async fn test_git_sync_workspace_not_bare() {
        let (origin_, _) = crate::test::repo_init(None, None);
        let origin = origin_.unwrap();

        let git_root_dir = TempDir::new().unwrap();
        let repo_dir = git_root_dir.path().join("10");
        let err_msg = format!("Invalid data at local {:?} - expected bare repo", repo_dir);
        let (_, repo) = crate::test::repo_init(None, Some(&repo_dir));
        let (_, _) = crate::test::commit(&repo, "some_file");

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
        assert_eq!(failed_sync.to_string(), err_msg);
    }

    #[async_std::test]
    async fn test_workspace_path_info_from_workspace_git_result_set() {
        let (td_, repo) = crate::test::repo_init(None, None);
        let (_, _) = crate::test::commit(&repo, "some_file");

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

        let git_pmr_accessor = GitPmrAccessor::new(
            &mock_backend,
            git_root.path().to_path_buf(),
            workspace,
        );

        match git_pmr_accessor.process_pathinfo(
            None,
            None,
            |git_pmr_accessor, result| {
                <WorkspacePathInfo>::from(&WorkspaceGitResultSet::new(&git_pmr_accessor.workspace, result))
            }
        ).await {
            Ok(workspace_path_info) => {
                assert_eq!(workspace_path_info.path, "".to_string());
                assert_eq!(workspace_path_info.description, Some("demo workspace 10".to_string()));
            }
            Err(_) => {
                unreachable!();
            }
        }
    }
}
