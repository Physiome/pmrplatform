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
        exposure::RawExposureCtrl,
    },
    error::PlatformError,
    platform::Platform,
};

impl<
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> Clone for ExposureCtrl<'_, '_, MCP, TMP> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> ExposureCtrl<'p, 'db, MCP, TMP>
where
    'p: 'db
{
    pub fn new(
        platform: &'p Platform<'db, MCP, TMP>,
        git_handle: GitHandle<'p, 'db, MCP>,
        exposure: ExposureRef<'db, MCP>,
    ) -> Self {
        Self(Arc::new(RawExposureCtrl {
            platform,
            git_handle,
            exposure,
            exposure_file_ctrls: Arc::new(Mutex::new(HashMap::new())),
        }))
    }

    pub async fn create_file(
        &'p self,
        workspace_file_path: &'db str,
    ) -> Result<
        ExposureFileCtrl<'p, 'db, MCP, TMP>,
        PlatformError
    > {
        // FIXME should fail with already exists if already created
        // quick failing here.
        let pathinfo = self.0.git_handle.pathinfo(
            Some(self.0.exposure.commit_id()),
            Some(workspace_file_path),
        )?;
        // path exists, so create the exposure file
        let efb: &dyn ExposureFileBackend = &self.0.platform.mc_platform;
        let exposure_file = self.0.platform.mc_platform.get_exposure_file(
            efb.insert(
                self.0.exposure.id(),
                workspace_file_path,
                None,
            ).await?
        ).await?;
        let exposure_file = ExposureFileCtrl::new(
            self.0.platform,
            self.clone(),
            exposure_file,
            pathinfo,
        );
        Ok(
            MutexGuard::map(
                self.0.exposure_file_ctrls.lock(),
                |efc| efc
                    .entry(workspace_file_path.to_string())
                    .or_insert(exposure_file)
            )
                .deref()
                .clone()
        )
    }

    pub fn ctrl_file(
        &'p self,
        exposure_file_ref: ExposureFileRef<'db, MCP>,
    ) -> Result<
        ExposureFileCtrl<'p, 'db, MCP, TMP>,
        PlatformError
    > {
        let workspace_file_path = exposure_file_ref
            .workspace_file_path()
            .to_string();

        // FIXME first verify that this entry is already present
        let pathinfo = self.0.git_handle.pathinfo(
            Some(self.0.exposure.commit_id()),
            Some(&workspace_file_path),
        )?;

        let exposure_file = ExposureFileCtrl::new(
            self.0.platform,
            self.clone(),
            exposure_file_ref,
            pathinfo,
        );
        Ok(
            MutexGuard::map(
                self.0.exposure_file_ctrls.lock(),
                |efc| efc
                    .entry(workspace_file_path.to_string())
                    .or_insert(exposure_file)
            )
                .deref()
                .clone()
        )
    }

    /// List all underlying files associated with the workspace at the
    /// commit id for this exposure.
    pub fn list_files(&self) -> Result<Vec<String>, PlatformError> {
        Ok(self.0.git_handle.files(Some(&self.0.exposure.commit_id()))?)
    }

    /// Returns a mapping of paths to actual files on the filesystem.
    pub fn map_files_fs(
        &self,
    ) -> Result<HashMap<String, String>, PlatformError> {
        let mut result = HashMap::new();
        let root = self.ensure_fs()?;
        self.0.git_handle
            .files(Some(&self.0.exposure.commit_id()))?
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
        let mut root = self.0.platform.data_root.join("exposure");
        root.push(self.0.exposure.id().to_string());
        root.push("files");
        if root.is_dir() {
            // assume the root is checked out already
            return Ok(root);
        }
        // TODO if pmrgit-fuse does get done, this checkout will become
        // very much optional
        // Also does this depend on a WorkspaceCtrl that manage this?
        std::fs::create_dir_all(&root)?;
        self.0.git_handle.checkout(Some(self.0.exposure.commit_id()), &root)?;
        Ok(root)
    }

    /// List all files that have a corresponding exposure file
    pub async fn list_exposure_files(&'p self) -> Result<Vec<&'db str>, PlatformError> {
        // FIXME this might not be accurate if we later create a new file.
        // using create_file after this call.
        Ok(self.0.exposure.files().await?
            .iter()
            .map(|f| f.workspace_file_path())
            .collect::<Vec<_>>()
        )
    }

    pub fn exposure(&self) -> &ExposureRef<'db, MCP> {
        &self.0.exposure
    }

}
