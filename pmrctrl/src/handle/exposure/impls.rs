use pmrcore::{
    exposure::traits::ExposureFileBackend,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};

use crate::{
    handle::{
        ExposureCtrl,
        ExposureFileCtrl,
    },
    error::PlatformError,
};

impl<
    'db,
    MCP: MCPlatform + Sync,
    TMP: TMPlatform + Sync,
> ExposureCtrl<'db, MCP, TMP> {
    pub async fn create_file<'repo>(
        &'db self,
        workspace_file_path: &'repo str,
    ) -> Result<ExposureFileCtrl<'db, 'repo, MCP, TMP>, PlatformError>
    where
        'db: 'repo
    {
        // quick failing here.
        let pathinfo = self.git_handle.pathinfo(
            Some(&self.inner.commit_id),
            Some(workspace_file_path),
        )?;
        // path exists, so create the exposure file
        let efb: &dyn ExposureFileBackend = &self.platform.mc_platform;
        let inner = efb.insert(
            self.inner.id,
            workspace_file_path,
            None,
        ).await?;
        let inner = efb.get_id(
            efb.insert(
                self.inner.id,
                workspace_file_path,
                None,
            ).await?
        ).await?;
        let platform = self.platform;
        // maybe return the id that would produce this from the platform?
        Ok(ExposureFileCtrl {
            platform,
            pathinfo,
            inner,
        })
    }

    /// List all files associated with this exposure.
    pub fn list_files(&self) -> Result<Vec<String>, PlatformError> {
        Ok(self.git_handle.files(Some(&self.inner.commit_id))?)
    }

    /// List the files that have a corresponding exposure file
    pub fn list_exposure_files(&self) -> Result<Vec<String>, PlatformError> {
        todo!()
    }

}
