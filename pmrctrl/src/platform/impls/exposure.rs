use pmrcore::{
    exposure::traits::{
        Exposure,
        ExposureBackend,
    },
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use crate::{
    error::PlatformError,
    handle::ExposureCtrl,
    platform::Platform,
};

impl<
    'a,
    MCP: MCPlatform + Sync,
    TMP: TMPlatform + Sync,
> Platform<'a, MCP, TMP> {
    /// Creates an exposure with all the relevant data validated.
    ///
    /// Returns a `ExposureCtrl` handle.
    pub async fn create_exposure(
        &'a self,
        workspace_id: i64,
        commit_id: &str,
    ) -> Result<ExposureCtrl<'a, MCP, TMP>, PlatformError> {
        // TODO verify that failing like so will be enough via thiserror
        let git_handle = self
            .repo_backend()
            .git_handle(workspace_id).await?;
        // TODO replace this with a more simple call? Like get_commit()?
        // calling pathinfo may be doing more than necessary work.
        {
            let _ = git_handle.pathinfo(Some(commit_id), None)?;
        }

        // workspace_id and commit verified, create the root exposure
        let eb: &dyn ExposureBackend = &self.mc_platform;
        let inner = self.mc_platform.get_exposure(
            eb.insert(
                workspace_id,
                None,
                commit_id,
                None,
            ).await?
        ).await?;
        let platform = self;
        Ok(ExposureCtrl {
            platform,
            git_handle,
            inner,
        })
    }

    pub async fn get_exposure(
        &'a self,
        id: i64,
    ) -> Result<ExposureCtrl<'a, MCP, TMP>, PlatformError> {
        let inner = self.mc_platform.get_exposure(id).await?;
        let git_handle = self
            .repo_backend()
            .git_handle(inner.workspace_id()).await?;
        let platform = self;
        Ok(ExposureCtrl {
            platform,
            git_handle,
            inner,
        })
    }
}
