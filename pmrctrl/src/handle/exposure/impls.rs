use parking_lot::{
    Mutex,
    MutexGuard,
};
use pmrcore::{
    exposure::{
        traits::{
            Exposure,
            ExposureFile,
            ExposureFileBackend,
        },
        ExposureRef,
        ExposureFileRef,
    },
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use pmrrepo::handle::GitHandle;
use std::{
    collections::HashMap,
    ops::Deref,
    sync::Arc,
};

use crate::{
    handle::{
        ExposureCtrl,
        ExposureFileCtrl,
    },
    error::PlatformError,
    platform::Platform,
};

impl<
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> ExposureCtrl<'db, MCP, TMP> {
    pub fn new(
        platform: &'db Platform<'db, MCP, TMP>,
        git_handle: GitHandle<'db, 'db, MCP>,
        exposure: ExposureRef<'db, MCP>,
    ) -> Self {
        Self {
            platform,
            git_handle,
            exposure,
            exposure_files: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_file(
        &'db self,
        workspace_file_path: &'db str,
    ) -> Result<ExposureFileCtrl<'db, MCP, TMP>, PlatformError> {
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

        let exposure_file = MutexGuard::map(
            self.exposure_files.lock(),
            |exposure_files| exposure_files
                .entry(workspace_file_path)
                .or_insert(exposure_file)
        );

        let platform = self.platform;
        // maybe return the id that would produce this from the platform?
        let result = Ok(ExposureFileCtrl {
            platform,
            pathinfo,
            exposure_file,
        });
        result
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
