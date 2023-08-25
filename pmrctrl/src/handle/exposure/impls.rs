use pmrcore::{
    exposure::traits::{
        Exposure,
        ExposureFile,
        ExposureFileBackend,
    },
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
            Some(self.inner.commit_id()),
            Some(workspace_file_path),
        )?;
        // path exists, so create the exposure file
        let efb: &dyn ExposureFileBackend = &self.platform.mc_platform;
        let inner = self.platform.mc_platform.get_exposure_file(
            efb.insert(
                self.inner.id(),
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
        Ok(self.git_handle.files(Some(&self.inner.commit_id()))?)
    }

    /// List the files that have a corresponding exposure file
    pub async fn list_exposure_files(&'db self) -> Result<Vec<String>, PlatformError> {
        // TODO don't use these inefficient abstractions
        // TODO make better abstraction that only pull from the column
        Ok(self.inner.files().await?
            .iter()
            // TODO cloning here is doubly inefficient
            .map(|f| f.workspace_file_path().to_string())
            .collect::<Vec<_>>()
        )
    }

}
