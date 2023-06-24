use async_recursion::async_recursion;
use futures::stream::StreamExt;
use futures::stream::futures_unordered::FuturesUnordered;
use std::{
    io::Write,
    ops::Deref,
};
use gix::{
    objs::{
        CommitRef,
        ObjectRef,
    },
    ThreadSafeRepository,
};
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
    GixError,
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
    pub repo: ThreadSafeRepository,
}

pub enum GitResultTarget<'a> {
    Object(ObjectRef<'a>),
    SubRepoPath {
        location: String,
        commit: String,
        path: &'a str,
    },
}

pub struct GitResult<'a> {
    pub repo: &'a ThreadSafeRepository,
    pub commit: CommitRef<'a>,
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
            .open(repo_dir)?;
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
        self.repo.to_thread_local().references()?.tags()?.map(|reference| {
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

}
