use futures::stream::{
    StreamExt,
    futures_unordered::FuturesUnordered,
};
use pmrmodel_base::{
    platform::Platform,
    workspace::{
        WorkspaceRef,
        traits::Workspace,
        traits::WorkspaceTagBackend,
    },
};
use std::{
    ops::Deref,
    path::PathBuf,
};

use crate::{
    backend::Backend,
    error::{
        ExecutionError,
        GixError,
        PmrRepoError,
    },
};
use super::{
    HandleW,
    HandleWR,
};

impl<'a, P: Platform + Sync> HandleW<'a, P> {
    pub(crate) fn new(
        backend: &'a Backend<P>,
        repo_root: PathBuf,
        workspace: WorkspaceRef<'a, P>,
    ) -> Self {
        let repo_dir = repo_root.join(workspace.id().to_string());
        Self { backend, repo_dir, workspace }
    }

    pub(crate) async fn sync_workspace(self) -> Result<HandleWR<'a, P>, PmrRepoError> {
        let ticket = self.workspace.begin_sync().await?;
        let repo_dir = &self.repo_dir.as_ref();
        let url = self.workspace.url();

        // currently this is the only implementation...
        // TODO eventually when a more generic trait that provides
        // common generic methods that will encapsulate all repo
        // implementation.
        match crate::git::fetch_or_clone(repo_dir, &url) {
            Ok(_) => {
                ticket.complete_sync().await?;
                let handle: HandleWR<'a, P> = self.try_into()?;
                handle.index_tags().await?;
                Ok(handle)
            }
            Err(e) => {
                ticket.fail_sync().await?;
                match e {
                    crate::git::error::FetchClone::Message(s) => Err(
                        ExecutionError::Synchronize {
                            workspace_id: self.workspace.id(),
                            remote: url.to_string(),
                            msg: s,
                        }.into()
                    ),
                    crate::git::error::FetchClone::Libgit2(e) => Err(e.into()),
                }
            }
        }
    }
}

impl<'a, P: Platform + Sync> TryFrom<HandleW<'a, P>> for HandleWR<'a, P> {
    type Error = GixError;

    fn try_from(item: HandleW<'a, P>) -> Result<Self, GixError> {
        let repo = gix::open::Options::isolated()
            .open_path_as_is(true)
            .open(&item.repo_dir)?
            .to_thread_local();
        Ok(Self {
            backend: item.backend,
            repo_dir: item.repo_dir,
            workspace: item.workspace,
            repo
        })
    }
}

impl<'a, P: Platform + Sync> HandleWR<'a, P> {
    pub(crate) fn new(
        backend: &'a Backend<P>,
        repo_root: PathBuf,
        workspace: WorkspaceRef<'a, P>,
    ) -> Result<Self, GixError> {
        let repo_dir = repo_root.join(workspace.id().to_string());
        let repo = gix::open::Options::isolated()
            .open_path_as_is(true)
            .open(&repo_dir)?
            .to_thread_local();
        Ok(Self { backend, repo_dir, workspace, repo })
    }

    pub async fn index_tags(&self) -> Result<(), GixError> {
        let platform = self.backend.db_platform;
        let workspace = &self.workspace;
        self.repo.references()?.tags()?
            .filter_map(|reference| {
                match reference {
                    Ok(tag) => {
                        let target = tag.target().id().to_hex().to_string();
                        match std::str::from_utf8(
                            tag.name().as_bstr().deref()
                        ) {
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
            })
            .map(|(name, oid)| async move {
                match WorkspaceTagBackend::index_workspace_tag(
                    platform,
                    workspace.id(),
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

    use mockall::predicate::*;
    use pmrmodel_base::workspace::{
        Workspace,
        WorkspaceSyncStatus,
    };
    use tempfile::TempDir;

    use crate::{
        backend::Backend,
        test::MockPlatform,
    };

    fn expect_workspace(
        platform: &mut MockPlatform,
        id: i64,
        url: &(impl std::fmt::Display + Sync + ?Sized),
    ) {
        let url = url.to_string();
        platform.expect_get_workspace_by_id()
            .times(1)
            .with(eq(id))
            .returning(move |_| Ok(Workspace {
                id: id,
                superceded_by_id: None,
                url: url.to_string(),
                description: None,
                long_description: None,
                created_ts: 1234567890,
                exposures: None,
            }));
    }

    #[async_std::test]
    async fn test_sync_workspace_empty() -> anyhow::Result<()> {
        let (td, _) = crate::test::repo_init(None, None, None)?;
        let td = td.as_ref().expect("tempdir created");
        let mut platform = MockPlatform::new();
        let wid = 1;
        let sid = 10;
        platform.expect_begin_sync()
            .times(1)
            .with(eq(wid))
            .returning(move |_| Ok(sid));
        platform.expect_complete_sync()
            .times(1)
            .with(eq(sid), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));
        expect_workspace(&mut platform, wid, td.path().to_str().unwrap());
        let repo_root = TempDir::new()?;
        let backend = Backend::new(&platform, repo_root.path().to_path_buf());
        backend.sync_workspace(wid).await?;
        Ok(())
    }

    #[async_std::test]
    async fn test_sync_workspace_with_index_tag() -> anyhow::Result<()> {
        let (td, _) = crate::test::repo_init(None, None, None)?;
        let td = td.as_ref().expect("tempdir created");
        // TODO use gix to tag?
        let repo = git2::Repository::open_bare(td)?;
        let oid = repo.head()?.target().expect("HEAD has a target");
        let obj = repo.find_object(oid, None)?;
        repo.tag_lightweight("new_tag", &obj, false)?;

        let mut platform = MockPlatform::new();
        let wid = 123;
        let sid = 1;
        platform.expect_begin_sync()
            .times(1)
            .with(eq(wid))
            .returning(move |_| Ok(sid));
        platform.expect_complete_sync()
            .times(1)
            .with(eq(sid), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));
        platform.expect_index_workspace_tag()
            .times(1)
            .withf(move |workspace_id: &i64, name: &str, commit_id: &str| {
                *workspace_id == wid && name == "refs/tags/new_tag" && commit_id == oid.to_string()
            })
            .returning(|_, _, _| Ok(1));
        expect_workspace(&mut platform, wid, td.path().to_str().unwrap());

        let repo_root = TempDir::new().unwrap();
        let backend = Backend::new(&platform, repo_root.path().to_path_buf());
        backend.sync_workspace(wid).await?;
        Ok(())
    }

    #[async_std::test]
    async fn test_sync_failure_invalid_remote() -> anyhow::Result<()> {
        // where remote couldn't be found or invalid.
        let td = TempDir::new().unwrap();
        let err_msg = format!(
            "ExecutionError: workspace `{0}`: failed to synchronize with remote `{1}`: \
            fail to clone: could not find repository from '{1}'; \
            class=Repository (6); code=NotFound (-3)", 2, td.path().to_str().unwrap());
        let mut platform = MockPlatform::new();
        let wid = 2;
        let sid = 3;
        platform.expect_begin_sync()
            .times(1)
            .with(eq(wid))
            .returning(move |_| Ok(sid));
        platform.expect_complete_sync()
            .times(1)
            .with(eq(sid), eq(WorkspaceSyncStatus::Error))
            .returning(|_, _| Ok(true));
        expect_workspace(&mut platform, wid, td.path().to_str().unwrap());

        let repo_root = TempDir::new().unwrap();
        let backend = Backend::new(&platform, repo_root.path().to_path_buf());
        let err = backend.sync_workspace(wid).await.unwrap_err();
        assert_eq!(err.to_string(), err_msg);
        Ok(())
    }

    #[async_std::test]
    async fn test_sync_failure_dropped_source() -> anyhow::Result<()> {
        let (td_, _) = crate::test::repo_init(None, None, None).unwrap();
        let td = td_.unwrap();
        let mut platform = MockPlatform::new();
        let wid = 42;
        let sid = 1;
        let url = td.path().to_str().unwrap().to_string();
        platform.expect_begin_sync()
            .times(1)
            .with(eq(wid))
            .returning(move |_| Ok(sid));
        platform.expect_complete_sync()
            .times(1)
            .with(eq(sid), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));
        expect_workspace(&mut platform, wid, &url);

        let repo_root = TempDir::new().unwrap();
        {
            let backend = Backend::new(&platform, repo_root.path().to_path_buf());
            assert!(backend.sync_workspace(wid).await.is_ok());
        }

        platform.checkpoint();

        // now verify that the failure to sync will generate the right error
        // when an originally working remote disappeared or errored.
        td.close()?;

        let sid = 2;
        platform.expect_begin_sync()
            .times(1)
            .with(eq(wid))
            .returning(move |_| Ok(sid));
        platform.expect_complete_sync()
            .times(1)
            .with(eq(sid), eq(WorkspaceSyncStatus::Error))
            .returning(|_, _| Ok(true));
        expect_workspace(&mut platform, wid, &url);

        let err_msg = format!(
            "ExecutionError: workspace `42`: failed to synchronize with \
            remote `{}`: unsupported URL protocol; class=Net (12)", url
        );
        let backend = Backend::new(&platform, repo_root.path().to_path_buf());
        let failed_sync = backend.sync_workspace(wid).await;
        assert_eq!(failed_sync.unwrap_err().to_string(), err_msg);
        Ok(())
    }

    #[async_std::test]
    async fn test_sync_workspace_not_bare() {
        let (origin_, _) = crate::test::repo_init(None, None, None).unwrap();
        let origin = origin_.unwrap();

        let repo_root = TempDir::new().unwrap();
        let repo_dir = repo_root.path().join("10");

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

        let mut platform = MockPlatform::new();
        platform.expect_begin_sync()
            .times(1)
            .with(eq(10))
            .returning(|_| Ok(1));
        platform.expect_complete_sync()
            .times(1)
            .with(eq(1), eq(WorkspaceSyncStatus::Error))
            .returning(|_, _| Ok(true));
        expect_workspace(&mut platform, 10, origin.path().to_str().unwrap());

        let backend = Backend::new(&platform, repo_root.path().to_path_buf());
        let failed_sync = backend.sync_workspace(10).await.unwrap_err();
        let err_msg = format!(
            "ExecutionError: workspace `10`: failed to synchronize with \
            remote `{}`: expected repo_dir be a bare repo",
            origin.path().display(),
        );
        assert_eq!(failed_sync.to_string(), err_msg);
    }

}
