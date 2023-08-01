use pmrmodel_base::{
    platform::Platform,
    workspace::{
        WorkspaceRef,
        traits::Workspace,
    },
};
use std::path::PathBuf;

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

    pub async fn sync_workspace(&'a self) -> Result<(), PmrRepoError> {
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
                Ok(())
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
        url: String,
    ) {
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
        let td = td
            .expect("tempdir created");
        let mut platform = MockPlatform::new();
        let workspace_id = 1;
        let sync_id = 10;
        platform.expect_begin_sync()
            .times(1)
            .with(eq(workspace_id))
            .returning(move |_| Ok(sync_id));
        platform.expect_complete_sync()
            .times(1)
            .with(eq(sync_id), eq(WorkspaceSyncStatus::Completed))
            .returning(|_, _| Ok(true));
        expect_workspace(
            &mut platform,
            workspace_id,
            td.path().to_str().unwrap().to_string()
        );
        let repo_root = TempDir::new()?;
        let backend = Backend::new(&platform, repo_root.path().to_path_buf());
        backend.sync_workspace(workspace_id).await?;
        Ok(())
    }
}
