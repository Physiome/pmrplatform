use pmrcore::{
    exposure::traits::ExposureBackend,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use crate::{
    error::PlatformError,
    platform::Platform,
};

impl<
    'a,
    MCP: MCPlatform + Sync,
    TMP: TMPlatform + Sync,
> Platform<'a, MCP, TMP> {
    /// Creates an exposure with all the relevant data validated.
    pub async fn create_exposure(
        &'a self,
        workspace_id: i64,
        commit_id: &str,
    ) -> Result<i64, PlatformError> {
        // TODO verify that failing like so will be enough via thiserror
        let repo_handle = self
            .repo_backend()
            .git_handle(workspace_id).await?;
        let info = repo_handle.pathinfo(Some(commit_id), None)?;

        // workspace_id and commit verified, create the root exposure
        let eb: &dyn ExposureBackend = &self.mc_platform;
        Ok(eb.insert(
            workspace_id,
            None,
            commit_id,
            None,
        ).await?)
    }
}
