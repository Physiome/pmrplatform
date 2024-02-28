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
use std::collections::HashMap;

use crate::{
    error::PlatformError,
    handle::ExposureCtrl,
    platform::Platform,
};

impl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> Platform<'db, MCP, TMP>
where
    'p: 'db
{
    /// Creates an exposure with all the relevant data validated.
    ///
    /// Returns a `ExposureCtrl` handle.
    pub async fn create_exposure(
        &'p self,
        workspace_id: i64,
        commit_id: &str,
    ) -> Result<ExposureCtrl<'p, 'db, MCP, TMP>, PlatformError> {
        // TODO verify that failing like so will be enough via thiserror
        let git_handle = self
            .repo_backend()
            .git_handle(workspace_id).await?;
        // TODO replace this with a more simple call? Like get_commit()?
        // calling pathinfo may be doing more than necessary work.
        {
            let _ = git_handle.pathinfo::<String>(Some(commit_id), None)?;
        }

        // workspace_id and commit verified, create the root exposure
        let eb: &dyn ExposureBackend = &self.mc_platform;
        let exposure = self.mc_platform.get_exposure(
            eb.insert(
                workspace_id,
                None,
                commit_id,
                None,
            ).await?
        ).await?;
        let platform = self;
        Ok(ExposureCtrl::new(
            platform,
            git_handle,
            exposure,
        ))
    }

    pub async fn get_exposure(
        &'p self,
        id: i64,
    ) -> Result<ExposureCtrl<'p, 'db, MCP, TMP>, PlatformError> {
        let exposure = self.mc_platform.get_exposure(id).await?;
        let git_handle = self
            .repo_backend()
            .git_handle(exposure.workspace_id()).await?;
        let platform = self;
        Ok(ExposureCtrl::new(
            platform,
            git_handle,
            exposure,
        ))
    }

    // Note that there is NO impls for returning an ExposureFileCtrl
    // directly as it tracks a GitHandleResult which requires GitHandle
    // held somewhere, which currently is typically from the Exposure.

}

mod task;
