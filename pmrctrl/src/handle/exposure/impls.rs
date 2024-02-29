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
    'p,
    'mcp_db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> ExposureCtrl<'p, 'mcp_db, MCP, TMP>
where
    'p: 'mcp_db
{
    pub fn new(
        platform: &'p Platform<'mcp_db, MCP, TMP>,
        git_handle: GitHandle<'p, 'mcp_db, MCP>,
        exposure: ExposureRef<'mcp_db, MCP>,
    ) -> Self {
        Self {
            platform,
            git_handle,
            exposure,
            exposure_file_ctrls: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_file(
        &'p self,
        workspace_file_path: &'mcp_db str,
    ) -> Result<
        impl Deref<Target=ExposureFileCtrl<'p, 'mcp_db, MCP, TMP>>,
        PlatformError
    > {
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
            self.exposure_file_ctrls.lock(),
            |exposure_files| {
                let platform = self.platform;
                let result = ExposureFileCtrl {
                    platform,
                    pathinfo,
                    exposure_file,
                };
                exposure_files
                    .entry(workspace_file_path.to_string())
                    .or_insert(result)
            }
        );

        Ok(exposure_file)
    }

    pub async fn ctrl_file(
        &'p self,
        exposure_file_ref: ExposureFileRef<'mcp_db, MCP>,
    ) -> Result<
        impl Deref<Target=ExposureFileCtrl<'p, 'mcp_db, MCP, TMP>>,
        PlatformError
    > {
        let workspace_file_path = exposure_file_ref
            .workspace_file_path()
            .to_string();
        let pathinfo = self.git_handle.pathinfo(
            Some(self.exposure.commit_id()),
            Some(workspace_file_path.clone()),
        )?;
        let exposure_file = MutexGuard::map(
            self.exposure_file_ctrls.lock(),
            |exposure_files| {
                let platform = self.platform;
                let result = ExposureFileCtrl {
                    platform,
                    pathinfo,
                    exposure_file: exposure_file_ref,
                };
                exposure_files
                    .entry(workspace_file_path)
                    .or_insert(result)
            }
        );

        Ok(exposure_file)
    }

    /// List all underlying files associated with the workspace at the
    /// commit id for this exposure.
    pub fn list_files(&self) -> Result<Vec<String>, PlatformError> {
        Ok(self.git_handle.files(Some(&self.exposure.commit_id()))?)
    }

    /// List all files that have a corresponding exposure file
    pub async fn list_exposure_files(&'p self) -> Result<Vec<&'mcp_db str>, PlatformError> {
        // FIXME this might not be accurate if we later create a new file.
        // using create_file after this call.
        Ok(self.exposure.files().await?
            .iter()
            .map(|f| f.workspace_file_path())
            .collect::<Vec<_>>()
        )
    }

}
