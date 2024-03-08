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
    path::PathBuf,
    sync::Arc,
};

use crate::{
    handle::{
        ExposureCtrl,
        ExposureFileCtrl,
    },
    handle::exposure_file::EFCData,
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
            efc_data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_file(
        &'p self,
        workspace_file_path: &'mcp_db str,
    ) -> Result<
        ExposureFileCtrl<'p, 'mcp_db, MCP, TMP>,
        PlatformError
    > {
        // FIXME should fail with already exists if already created
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
        let exposure_file = ExposureFileCtrl {
            platform: self.platform,
            exposure: self,
            data: MutexGuard::map(
            self.efc_data.lock(),
                |efc_datum| efc_datum
                    .entry(workspace_file_path.to_string())
                    .or_insert(EFCData {
                        pathinfo,
                        exposure_file,
                    })
            ),
        };

        Ok(exposure_file)
    }

    pub async fn ctrl_file(
        &'p self,
        exposure_file_ref: ExposureFileRef<'mcp_db, MCP>,
    ) -> Result<
        ExposureFileCtrl<'p, 'mcp_db, MCP, TMP>,
        PlatformError
    > {
        let workspace_file_path = exposure_file_ref
            .workspace_file_path()
            .to_string();

        // FIXME first verify that this entry is already present
        let pathinfo = self.git_handle.pathinfo(
            Some(self.exposure.commit_id()),
            Some(workspace_file_path.clone()),
        )?;

        let exposure_file = ExposureFileCtrl {
            platform: self.platform,
            exposure: self,
            data: MutexGuard::map(
            self.efc_data.lock(),
                |efc_datum| efc_datum
                    .entry(workspace_file_path.to_string())
                    .or_insert(EFCData {
                        pathinfo,
                        exposure_file: exposure_file_ref,
                    })
            ),
        };

        Ok(exposure_file)
    }

    /// List all underlying files associated with the workspace at the
    /// commit id for this exposure.
    pub fn list_files(&self) -> Result<Vec<String>, PlatformError> {
        Ok(self.git_handle.files(Some(&self.exposure.commit_id()))?)
    }

    /// Returns a mapping of paths to actual files on the filesystem.
    pub fn map_files_fs(
        &self,
    ) -> Result<HashMap<String, String>, PlatformError> {
        let mut result = HashMap::new();
        let root = self.ensure_fs()?;
        self.git_handle
            .files(Some(&self.exposure.commit_id()))?
            .iter()
            .for_each(|path| {
                result.insert(
                    path.to_string(),
                    // TODO maybe split and join? not sure if backslashes
                    // will make a difference on Windows, but we don't care
                    // about Windows for the prototype.
                    root.join(path).display().to_string(),
                );
            });
        Ok(result)
    }

    /// This ensures there is filesystem level access to the underlying
    /// files for this exposure (backed by the relevant workspace at the
    /// specified commit_id).
    ///
    /// Currently, the implementation is done here directly, but in the
    /// future this should be delegated to the platform as it should be
    /// able to determine what to offer from configuration, e.g. via a
    /// simple checkout like it's currently done, through a central
    /// location offered via fuse or distributed via some other manner.
    pub fn ensure_fs(
        &self,
    ) -> Result<PathBuf, PlatformError> {
        let mut root = self.platform.data_root.join("exposure");
        root.push(self.exposure.id().to_string());
        root.push("files");
        if root.is_dir() {
            // assume the root is checked out already
            return Ok(root);
        }
        std::fs::create_dir_all(&root)?;
        self.git_handle.checkout(Some(self.exposure.commit_id()), &root)?;
        Ok(root)
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

    pub fn exposure(&self) -> &ExposureRef<'mcp_db, MCP> {
        &self.exposure
    }

}
