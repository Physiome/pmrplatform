use pmrcore::{
    workspace::{
        WorkspaceRef,
        traits::Workspace as _,
    },
};
use std::path::PathBuf;

use crate::{
    backend::Backend,
    error::{
        ExecutionError,
        PmrRepoError,
    },
};
use super::{
    Handle,
    GitHandle,
};

impl<'handle> Handle<'handle> {
    pub(crate) fn new(
        backend: &'handle Backend,
        repo_root: PathBuf,
        workspace: WorkspaceRef<'handle>,
    ) -> Self {
        let repo_dir = repo_root.join(workspace.id().to_string());
        Self { backend, repo_dir, workspace }
    }

    pub(crate) async fn sync_workspace(
        self,
    ) -> Result<GitHandle<'handle>, PmrRepoError> {
        let ticket = self.workspace.begin_sync().await?;
        let repo_dir = &self.repo_dir.as_ref();
        let url = self.workspace.url();

        // currently this is the only implementation...
        // TODO eventually when a more generic trait that provides
        // common generic methods that will encapsulate all repo
        // implementation.
        match super::git::util::fetch_or_clone(repo_dir, &url) {
            Ok(_) => {
                ticket.complete_sync().await?;
                let handle: GitHandle<'handle> = self.into();
                handle.index_tags().await?;
                Ok(handle)
            }
            Err(e) => {
                ticket.fail_sync().await?;
                match e {
                    super::git::error::FetchClone::Message(s) => Err(
                        ExecutionError::Synchronize {
                            workspace_id: self.workspace.id(),
                            remote: url.to_string(),
                            msg: s,
                        }.into()
                    ),
                    super::git::error::FetchClone::Libgit2(e) => Err(e.into()),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use pmrcore::{
        repo::{
            PathObjectInfo,
            RemoteInfo,
        },
        workspace::{
            Workspace,
            WorkspaceSyncStatus,
        },
    };
    use std::sync::Arc;
    use tempfile::TempDir;

    use test_pmr::core::MockPlatform;

    use crate::{
        backend::Backend,
        handle::GitResultTarget,
    };
    use super::*;

    fn expect_workspace(
        platform: &mut MockPlatform,
        id: i64,
        url: &(impl std::fmt::Display + Sync + ?Sized),
    ) {
        let url = url.to_string();
        platform.expect_workspace_get_workspace_by_id()
            .times(1)
            .with(eq(id))
            .returning(move |_| Ok(Workspace {
                id: id,
                superceded_by_id: None,
                url: url.to_string(),
                description: Some(format!("Workspace {id}")),
                long_description: None,
                created_ts: 1234567890,
                exposures: None,
            }));
    }

    #[async_std::test]
    async fn test_sync_workspace_empty() -> anyhow::Result<()> {
        let (td, _) = test_pmr::repo::repo_init(None, None, None)?;
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
        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
        backend.sync_workspace(wid).await?;
        Ok(())
    }

    #[async_std::test]
    async fn test_sync_workspace_with_index_tag() -> anyhow::Result<()> {
        let (td, _) = test_pmr::repo::repo_init(None, None, None)?;
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
        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
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
        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
        let err = backend.sync_workspace(wid).await.unwrap_err();
        assert_eq!(err.to_string(), err_msg);
        Ok(())
    }

    #[async_std::test]
    async fn test_sync_failure_dropped_source() -> anyhow::Result<()> {
        let (td_, _) = test_pmr::repo::repo_init(None, None, None).unwrap();
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
        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
        assert!(backend.sync_workspace(wid).await.is_ok());

        let mut platform = MockPlatform::new();
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

        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
        let failed_sync = backend.sync_workspace(wid).await;
        assert_eq!(failed_sync.unwrap_err().to_string(), err_msg);
        Ok(())
    }

    #[async_std::test]
    async fn test_sync_workspace_not_bare() -> anyhow::Result<()> {
        let (origin_, _) = test_pmr::repo::repo_init(None, None, None).unwrap();
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
        test_pmr::repo::init_empty_commit(&repo, None).unwrap();
        test_pmr::repo::commit(&repo, vec![("some_file", "")]).unwrap();

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

        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
        let failed_sync = backend.sync_workspace(10).await.unwrap_err();
        let err_msg = format!(
            "ExecutionError: workspace `10`: failed to synchronize with \
            remote `{}`: expected repo_dir be a bare repo",
            origin.path().display(),
        );
        assert_eq!(failed_sync.to_string(), err_msg);
        Ok(())
    }

    #[async_std::test]
    async fn test_workspace_path_info_from_workspace_git_result() -> anyhow::Result<()> {
        let (td_, repo) = test_pmr::repo::repo_init(None, None, None).unwrap();
        test_pmr::repo::commit(&repo, vec![("some_file", "")]).unwrap();

        let td = td_.unwrap();

        let repo_root = TempDir::new().unwrap();
        let mut platform = MockPlatform::new();
        platform.expect_begin_sync()
            .times(1)
            .with(eq(10))
            .returning(|_| Ok(10));
        platform.expect_complete_sync()
            .times(1)
            .with(eq(10), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));
        expect_workspace(&mut platform, 10, td.path().to_str().unwrap());
        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
        assert!(backend.sync_workspace(10).await.is_ok());

        let mut platform = MockPlatform::new();
        expect_workspace(&mut platform, 10, td.path().to_str().unwrap());
        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
        let handle = backend.git_handle(10).await?;
        let result = handle.pathinfo::<String>(None, None).unwrap();
        assert_eq!(result.path(), Some(""));
        assert_eq!(result.workspace().description(), Some("Workspace 10"));
        Ok(())
    }

    #[async_std::test]
    async fn test_workspace_submodule_access() -> anyhow::Result<()> {
        let (
            repo_root,
            _, // (import1, import1_oids),
            _, // (import2, import2_oids),
            _, // (repodata, repodata_oids)
        ) = test_pmr::repo::create_repodata();

        let mut platform = MockPlatform::new();
        platform.expect_workspace_list_workspace_by_url()
            .times(1)
            .with(eq("http://models.example.com/w/import2"))
            .returning(|_| Ok([Workspace {
                id: 2,
                url: "http://models.example.com/w/import2".to_string(),
                description: Some("Workspace 2".to_string()),
                superceded_by_id: None,
                long_description: None,
                created_ts: 1234567890,
                exposures: None,
            }].into()));
        expect_workspace(&mut platform, 3, "http://models.example.com/w/repodata");
        expect_workspace(&mut platform, 2, "http://models.example.com/w/import2");

        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
        let handle = backend.git_handle(3).await?;
        let pathinfo = handle.pathinfo(
            Some("557ee3cb13fb421d2bd6897615ae95830eb427c8"),
            Some("ext/import1/README"),
        ).unwrap();

        assert_eq!(
            pathinfo.path(),
            Some("ext/import1/README"));
        let Some(GitResultTarget::RemoteInfo(target)) = pathinfo.target else {
            unreachable!()
        };
        assert_eq!(
            target,
            RemoteInfo {
                location: "http://models.example.com/w/import1"
                    .to_string(),
                commit: "01b952d14a0a33d22a0aa465fe763e5d17b15d46"
                    .to_string(),
                subpath: "README".to_string(),
                path: "ext/import1/README".to_string(),
            },
        );

        let pathinfo = handle.pathinfo(
            Some("c4d735e5a305559c1cb0ce8de4c25ed5c3f4f263"),
            Some("ext/import2/import1/if1"),
        ).unwrap();
        assert_eq!(
            pathinfo.path(),
            Some("ext/import2/import1/if1"),
        );
        let Some(GitResultTarget::RemoteInfo(target)) = pathinfo.target else {
            unreachable!()
        };
        assert_eq!(
            target,
            RemoteInfo {
                location: "http://models.example.com/w/import2"
                    .to_string(),
                commit: "0ab8a26a0e85a033bea0388216667d83cc0dc1dd"
                    .to_string(),
                subpath: "import1/if1".to_string(),
                path: "ext/import2/import1/if1".to_string(),
            },
        );

        let mut buffer = <Vec<u8>>::new();
        let readme_result = handle.pathinfo(
            Some("557ee3cb13fb421d2bd6897615ae95830eb427c8"),
            Some("README"),
        )?;
        assert_eq!(
            readme_result.stream_blob(&mut buffer).await?,
            22,
        );
        assert_eq!(
            std::str::from_utf8(&buffer).unwrap(),
            "A simple readme file.\n",
        );

        let mut buffer = <Vec<u8>>::new();
        let import2_result = handle.pathinfo(
            Some("a4a04eed5e243e3019592579a7f6eb950399f9bf"),
            Some("ext/import2/if2"),
        )?;
        assert_eq!(
            import2_result.stream_blob(&mut buffer).await?,
            4,
        );
        assert_eq!(
            std::str::from_utf8(&buffer).unwrap(),
            "if2\n",
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_workspace_loginfo() -> anyhow::Result<()> {
        let (
            repo_root,
            _, // (import1, import1_oids),
            _, // (import2, import2_oids),
            (_, repodata_oids),
        ) = test_pmr::repo::create_repodata();

        let mut platform = MockPlatform::new();
        expect_workspace(&mut platform, 3, "http://models.example.com/w/repodata");

        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
        let handle = backend.git_handle(3).await?;
        let logs = handle.loginfo_detailed(None, None, None, true).unwrap();
        assert_eq!(
            repodata_oids.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>(),
            logs.entries.iter()
                .map(|x| x.commit_id.clone())
                .rev()
                .skip(1)  // skip the initial commit
                .collect::<Vec<_>>(),
        );
        assert_eq!(logs, serde_json::from_str(r#"{
          "entries": [
            {
              "commit_id": "8ae6e9af37c8bd78614545d0ab807348fc46dcab",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666780,
              "message": "updating dir1"
            },
            {
              "commit_id": "c4d735e5a305559c1cb0ce8de4c25ed5c3f4f263",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666770,
              "message": "fixing import1"
            },
            {
              "commit_id": "a4a04eed5e243e3019592579a7f6eb950399f9bf",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666760,
              "message": "bumping import2, breaking import1"
            },
            {
              "commit_id": "502b18ac456c8e475f731cbfe568fd6eb1177327",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666750,
              "message": "bumping import2"
            },
            {
              "commit_id": "965ccc1276832489c69b680b49874a6e1dc1743b",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666740,
              "message": "adding import2"
            },
            {
              "commit_id": "27be7efbe5fcccda5ee6ca00ef96834f592139a5",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666730,
              "message": "bumping import1"
            },
            {
              "commit_id": "e931905807563cb5353958e865d72fed12dccd4f",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666720,
              "message": "adding some files"
            },
            {
              "commit_id": "557ee3cb13fb421d2bd6897615ae95830eb427c8",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666710,
              "message": "adding import1"
            },
            {
              "commit_id": "9f02f69509110e7235e4bb9f50e235a246ae9f5c",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1666666700,
              "message": "Initial commit of repodata"
            },
            {
              "commit_id": "e55a6e1058fe4caf81e5cdfe806a3f86e1b94fb2",
              "author": "user <user@example.com>",
              "committer": "user <user@example.com>",
              "commit_timestamp": 1654321000,
              "message": "initial commit"
            }
          ]
        }"#)?);

        // for a specific path
        assert_eq!(
            [
                "27be7efbe5fcccda5ee6ca00ef96834f592139a5",
                "9f02f69509110e7235e4bb9f50e235a246ae9f5c",
            ],
            handle
                .loginfo(None, Some("file1"), None)?
                .entries.iter()
                .map(|x| x.commit_id.to_string())
                .collect::<Vec<_>>()
                .as_ref(),
        );
        assert_eq!(
            [
                "c4d735e5a305559c1cb0ce8de4c25ed5c3f4f263",
                "a4a04eed5e243e3019592579a7f6eb950399f9bf",
                "27be7efbe5fcccda5ee6ca00ef96834f592139a5",
                "557ee3cb13fb421d2bd6897615ae95830eb427c8",
            ],
            handle
                .loginfo(None, Some("ext/import1"), None)?
                .entries.iter()
                .map(|x| x.commit_id.to_string())
                .collect::<Vec<_>>()
                .as_ref(),
        );
        assert_eq!(
            0,
            handle
                .loginfo(None, Some("no/such/path"), None)?
                .entries.iter()
                .map(|x| x.commit_id.to_string())
                .collect::<Vec<_>>()
                .len(),
        );

        // from both a path and commit
        assert_eq!(
            [
                "27be7efbe5fcccda5ee6ca00ef96834f592139a5",
                "557ee3cb13fb421d2bd6897615ae95830eb427c8",
            ],
            handle
                .loginfo(
                    Some("502b18ac456c8e475f731cbfe568fd6eb1177327"),
                    Some("ext/import1"),
                    None,
                )?
                .entries.iter()
                .map(|x| x.commit_id.to_string())
                .collect::<Vec<_>>()
                .as_ref(),
        );

        // from both a path and commit and count
        assert_eq!(
            [
                "27be7efbe5fcccda5ee6ca00ef96834f592139a5",
            ],
            handle
                .loginfo(
                    Some("502b18ac456c8e475f731cbfe568fd6eb1177327"),
                    Some("ext/import1"),
                    Some(1),
                )?
                .entries.iter()
                .map(|x| x.commit_id.to_string())
                .collect::<Vec<_>>()
                .as_ref(),
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_workspace_files() -> anyhow::Result<()> {
        let (
            repo_root,
            _, // (import1, import1_oids),
            _, // (import2, import2_oids),
            _, // (repodata, repodata_oids)
        ) = test_pmr::repo::create_repodata();

        let mut platform = MockPlatform::new();
        expect_workspace(&mut platform, 3, "http://models.example.com/w/repodata");
        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
        let handle = backend.git_handle(3).await?;

        assert_eq!(
            handle.files(None)?,
            [
                ".gitmodules",
                "README",
                "dir1/file1",
                "dir1/file2",
                "dir1/nested/file_a",
                "dir1/nested/file_b",
                "dir1/nested/file_c",
                "ext/import1",
                "ext/import2",
                "file1",
                "file2",
            ]
        );
        assert_eq!(
            handle.files(
                Some("9f02f69509110e7235e4bb9f50e235a246ae9f5c"))?,
            [
                "README",
                "file1",
            ]
        );

        let pathinfo = handle.pathinfo(None, Some("dir1/nested/file_a"))?;
        assert_eq!(pathinfo.path(), Some("dir1/nested/file_a"));

        Ok(())
    }

    #[async_std::test]
    async fn test_workspace_pathobjectinfo() -> anyhow::Result<()> {
        let (
            repo_root,
            _, // (import1, import1_oids),
            _, // (import2, import2_oids),
            _, // (repodata, repodata_oids)
        ) = test_pmr::repo::create_repodata();

        let mut platform = MockPlatform::new();
        expect_workspace(&mut platform, 3, "http://models.example.com/w/repodata");
        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
        let handle = backend.git_handle(3).await?;

        // TODO do a test for the Object<'repo> conversion, but currently
        // this is no longer "reachable" through the pmrrepo API as most
        // are converted to the detached version for Send + Sync.
        let pathinfo = handle.pathinfo(Some("8ae6e9af37c8bd78614545d0ab807348fc46dcab"), None)?;
        match <Option<PathObjectInfo>>::from(&pathinfo) {
            Some(PathObjectInfo::TreeInfo(tree_info)) => {
                assert_eq!(tree_info.filecount, 6);
                assert_eq!(tree_info.entries[0], serde_json::from_str(r#"{
                    "filemode": "100644",
                    "id": "e5571ee030566a854f5f862b94138aed1c48918b",
                    "kind": "blob",
                    "name": ".gitmodules"
                }"#)?);
            }
            _ => unreachable!()
        }

        Ok(())
    }

    #[async_std::test]
    async fn test_workspace_borrow_move() -> anyhow::Result<()> {
        let (
            repo_root,
            _, // (import1, import1_oids),
            _, // (import2, import2_oids),
            _, // (repodata, repodata_oids)
        ) = test_pmr::repo::create_repodata();
        let mut platform = MockPlatform::new();
        expect_workspace(&mut platform, 3, "http://models.example.com/w/repodata");
        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
        let handle = backend.git_handle(3).await?;
        {
            let _ = handle.pathinfo::<String>(None, None);
        }
        // TODO figure out why we wanted a partial move here?
        // let _ = handle.workspace.into_inner();
        // will not work any more as the underlying became private...
        let _ = handle.workspace();
        Ok(())
    }

    #[async_std::test]
    async fn test_checkout() -> anyhow::Result<()> {
        let (
            repo_root,
            _, // (import1, import1_oids),
            _, // (import2, import2_oids),
            _, // (repodata, repodata_oids)
        ) = test_pmr::repo::create_repodata();
        let mut platform = MockPlatform::new();
        expect_workspace(&mut platform, 1, "http://models.example.com/");
        let backend = Backend::new(Arc::new(platform), repo_root.path().to_path_buf());
        let handle = backend.git_handle(1).await?;

        let checkout_root = TempDir::new()?;
        let target = checkout_root.path();
        handle.checkout(
            Some("42845247d1a2af1bf5a0f09c85e254ba78992c2f"),
            target.as_ref(),
        )?;
        let dir_branch = target.join("branch");
        let dir_leaf = dir_branch.join("leaf");
        assert!(dir_branch.is_dir());
        assert!(!dir_leaf.is_dir());

        let checkout_root = TempDir::new()?;
        let target = checkout_root.path();
        handle.checkout(
            None,
            target.as_ref(),
        )?;
        let dir_branch = target.join("branch");
        let dir_leaf = dir_branch.join("leaf");
        assert!(dir_branch.is_dir());
        assert!(dir_leaf.is_dir());

        Ok(())
    }

}
