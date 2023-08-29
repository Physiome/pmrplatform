use async_trait::async_trait;
use pmrcore::{
    error::ValueError,
    exposure::{
        ExposureFileRefs,
        traits::{
            Exposure,
            ExposureFile,
            ExposureFileBackend,
        },
    },
    platform::{
        MCPlatform,
        TMPlatform,
    },
    workspace::WorkspaceRef,
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
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
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
            Some(self.exposure.commit_id()),
            Some(workspace_file_path),
        )?;
        // path exists, so create the exposure file
        let efb: &dyn ExposureFileBackend = &self.platform.mc_platform;
        let exposure_file = self.platform.mc_platform.get_exposure_file(
            efb.insert(
                self.exposure.id(),
                workspace_file_path,
                None,
            ).await?
        ).await?;
        let platform = self.platform;
        // maybe return the id that would produce this from the platform?
        Ok(ExposureFileCtrl {
            platform,
            pathinfo,
            exposure_file,
        })
    }

    /// List all files associated with this exposure.
    pub fn list_files(&self) -> Result<Vec<String>, PlatformError> {
        Ok(self.git_handle.files(Some(&self.exposure.commit_id()))?)
    }

    /// List the files that have a corresponding exposure file
    pub async fn list_exposure_files(&'db self) -> Result<Vec<String>, PlatformError> {
        // TODO don't use these inefficient abstractions
        // TODO make better abstraction that only pull from the column
        Ok(self.exposure.files().await?
            .iter()
            // TODO cloning here is doubly inefficient
            .map(|f| f.workspace_file_path().to_string())
            .collect::<Vec<_>>()
        )
    }

}

/*
// It would have been nice if this was possible, but due to how the ctrl
// type holds a git handle which references the Repository which is
// !Sync, the following is not possible.  The inner field was then
// converted to a pub field.

#[async_trait]
impl<'db, MCP: MCPlatform + Sized + Sync, TMP: TMPlatform + Sized + Sync>
    Exposure<'db, ExposureFileRefs<'db, MCP>, WorkspaceRef<'db, MCP>>
for ExposureCtrl<'db, MCP, TMP> {
    fn id(&self) -> i64 {
        self.inner.id()
    }
    fn workspace_id(&self) -> i64 {
        self.inner.workspace_id()
    }
    fn workspace_tag_id(&self) -> Option<i64> {
        self.inner.workspace_tag_id()
    }
    fn commit_id(&self) -> &str {
        self.inner.commit_id()
    }
    fn created_ts(&self) -> i64 {
        self.inner.created_ts()
    }
    fn default_file_id(&self) -> Option<i64> {
        self.inner.default_file_id()
    }
    async fn files(&'db self) -> Result<&'db ExposureFileRefs<'db, MCP>, ValueError> {
        self.inner.files()
    }
    async fn workspace(&'db self) -> Result<&'db WorkspaceRef<'db, MCP>, ValueError> {
        self.inner.workspace()
    }
}
*/
