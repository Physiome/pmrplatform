use parking_lot::{
    Mutex,
    MutexGuard,
};
use pmrcore::{
    exposure::{
        profile::ExposureFileProfile,
        traits::{
            Exposure,
            ExposureFile,
            ExposureFileBackend,
        },
        ExposureRef,
        ExposureFileRef,
    },
};
use pmrmodel::model::profile::UserPromptGroupRefs;
use pmrrepo::handle::GitHandle;
use std::{
    collections::HashMap,
    ops::Deref,
    path::PathBuf,
    sync::{
        Arc,
        OnceLock,
    },
};

use crate::{
    handle::{
        EFViewTaskTemplatesCtrl,
        ExposureCtrl,
        ExposureFileCtrl,
        ExposureFileViewCtrl,
        exposure::RawExposureCtrl,
    },
    error::{
        CtrlError,
        PlatformError,
    },
    platform::Platform,
};

impl Clone for ExposureCtrl<'_> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<'p> ExposureCtrl<'p> {
    pub fn new(
        platform: &'p Platform,
        git_handle: GitHandle<'p>,
        exposure: ExposureRef<'p>,
    ) -> Self {
        Self(Arc::new(RawExposureCtrl {
            platform,
            git_handle,
            exposure,
            exposure_file_ctrls: Arc::new(Mutex::new(HashMap::new())),
            efvttcs: OnceLock::new(),
        }))
    }

    /// Create a file under the specified path.  This path must exist at
    /// the underlying Git repository.
    pub async fn create_file(
        &'p self,
        workspace_file_path: &'p str,
    ) -> Result<
        ExposureFileCtrl<'p>,
        PlatformError
    > {
        // FIXME should fail with already exists if already created
        // quick failing here.
        let pathinfo = self.0.git_handle.pathinfo(
            Some(self.0.exposure.commit_id()),
            Some(workspace_file_path),
        )?;
        // path exists, so create the exposure file
        let mcp = self.0.platform.mc_platform.as_ref();
        let exposure_file = self.0.platform.mc_platform.get_exposure_file(
            ExposureFileBackend::insert(
                mcp,
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

    /// Control a file using the ExposureFileRef.
    ///
    /// The underlying path must exist.
    pub fn ctrl_file(
        &'p self,
        exposure_file_ref: ExposureFileRef<'p>,
    ) -> Result<
        ExposureFileCtrl<'p>,
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

        // FIXME should the ref's platform be checked for validity?
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

    /// Acquire a ExposureFileCtrl using the exact workspace_file_path
    /// being provided.
    pub async fn ctrl_path(
        &'p self,
        workspace_file_path: impl AsRef<str> + ToString,
    ) -> Result<
        ExposureFileCtrl<'p>,
        PlatformError
    > {
        // quick failing here.
        let pathinfo = self.0.git_handle.pathinfo(
            Some(self.0.exposure.commit_id()),
            Some(workspace_file_path.as_ref()),
        )?;
        // FIXME What if pathinfo is a tree?  There is currently no way
        // to provide a ctrl for that.

        // path exists, so create the exposure file
        // TODO need to check if already present in exposure_file_ctrls
        let exposure_file = self.0.platform.mc_platform.get_exposure_file_by_id_path(
            self.0.exposure.id(),
            workspace_file_path.as_ref(),
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

    /// Acquire a ExposureFileCtrl using the provided id
    pub async fn ctrl_id(
        &'p self,
        id: i64,
    ) -> Result<ExposureFileCtrl<'p>, PlatformError> {
        self.ctrl_file(self.0.platform
            .mc_platform
            .get_exposure_file(id)
            .await?)
    }

    /// Resolve a ExposureFileCtrl using the workspace_file_path being
    /// provided, while attempting to extract a potential viewstr suffix
    /// that might be part of this path.
    ///
    /// Returns some tuple containing `ExposureFileCtrl` and some
    /// `viewstr`, the `viewstr` is `None` if the path is an exact
    /// match, otherwise a trailing slash on the same path will set it
    /// to some empty string.
    ///
    /// e.g. given an ExposureFile exists at path `dir/file`, providing
    /// path as `dir/file` will result in the identical outcome as the
    /// underlying `ctrl_path`, with viewstr set to `None`; `dir/file/`
    /// will result in the same ctrl while viewstr set to `Some("")`.
    ///
    /// Providing path as `dir/file/view/subpath` will also return the
    /// ctrl at `dir/file` with the viewstr specified as `view/subpath`.
    /// The viewstr will generally resolve into a view identified by the
    /// first fragment while all subsequent fragments are treated as the
    /// subpath within that view.
    pub async fn resolve_file_viewstr(
        &'p self,
        path: &'p str,
    ) -> Result<(ExposureFileCtrl<'p>, Option<&'p str>), CtrlError> {
        // TODO there should be a companion method `resolve_file_view` that
        // will resolve the actual file and view in one shot?
        for (idx, c) in [(path.len(), "")].into_iter()
            .chain(path.rmatch_indices('/'))
        {
            let (path, viewstr) = (&path[0..idx], &path[idx + c.len()..]);
            if path.chars().last() == Some('/') && c == "" {
                continue
            }
            log::trace!("checking path={path:?} viewstr={viewstr:?}");
            match self.ctrl_path(path).await {
                Ok(ctrl) => return Ok((ctrl, (c == "/").then_some(viewstr))),
                // assume BackendError here means the expected database
                // entry for ExposureFile is missing
                Err(PlatformError::BackendError(_)) => return Err(
                    CtrlError::EFCNotFound(path.to_string()).into()),
                Err(_) => continue,
            }
        }
        // TODO it may be useful to disambiguate _which_ failure happened,
        // e.g. if path found but no exposure file.
        Err(CtrlError::UnknownPath(path.to_string()).into())
    }

    pub async fn resolve_file_view(
        &'p self,
        path: &'p str,
    ) -> (Result<ExposureFileCtrl<'p>, CtrlError>, Result<ExposureFileViewCtrl<'p>, CtrlError>) {
        match self.resolve_file_viewstr(path).await {
            Ok((efc, Some(viewstr))) => (Ok(efc.clone()), efc.resolve_view_by_viewstr(viewstr).await),
            Ok((efc, None)) => (Ok(efc), Err(CtrlError::None)),
            Err(e) => (Err(e), Err(CtrlError::None)),
        }
    }

    /// List all underlying files associated with the workspace at the
    /// commit id for this exposure.
    pub fn list_git_files(&self) -> Result<Vec<String>, PlatformError> {
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
    pub async fn list_exposure_files(&'p self) -> Result<Vec<&'p str>, PlatformError> {
        // FIXME this might not be accurate if we later create a new file.
        // using create_file after this call.
        Ok(self.0.exposure.files().await?
            .iter()
            .map(|f| f.workspace_file_path())
            .collect::<Vec<_>>()
        )
    }

    /// Pairs all underlying files associated with the workspace at the
    /// commit id for this exposure, with an additional flag denoting if
    /// the path has an exposure file.
    pub async fn pair_files_info(
        &'p self,
    ) -> Result<Vec<(String, bool)>, PlatformError> {
        // Ok(self.0.git_handle.files(Some(&self.0.exposure.commit_id()))?)
        let mut files = self.list_git_files()?;
        files.sort_unstable();
        let mut exposure_files = self.list_exposure_files().await?;
        exposure_files.sort_unstable();
        let mut exposure_files = exposure_files.into_iter().peekable();

        Ok(files.into_iter()
            .map(|file| {
                if exposure_files.peek() == Some(&(file.as_ref())) {
                    exposure_files.next();
                    (file, true)
                } else {
                    (file, false)
                }
            })
            .collect::<Vec<_>>()
        )
    }

    /// This returns a list of parings of all files from the underlying
    /// git repository and any `ExposureFileCtrl` that associates with
    /// that path.
    pub async fn pair_files_efcs(
        &'p self,
    ) -> Result<Vec<(String, Option<ExposureFileCtrl<'p>>)>, PlatformError> {
        let mut result = Vec::new();
        for (path, cond) in self.pair_files_info().await?.into_iter() {
            let item = if cond {
                Some(self.ctrl_path(&path).await?)
            } else {
                None
            };
            result.push((path, item));
        }
        Ok(result)
    }

    /// This returns a list of parings of all files from the underlying
    /// git repository and any `ExposureFileCtrl` that associates with
    /// that path.
    pub async fn pair_files_efvttcs(
        &'p self,
    ) -> Result<&'p [(String, Option<EFViewTaskTemplatesCtrl<'p>>)], PlatformError> {
        Ok(match self.0.efvttcs.get() {
            Some(efvttcs) => efvttcs,
            None => {
                let efcs = self.pair_files_efcs().await?;
                let mut efvttcs = Vec::new();
                for (path, value) in efcs.into_iter() {
                    let item = if let Some(efc) = value {
                        Some(efc.try_into_vttc().await?)
                    } else {
                        None
                    };
                    efvttcs.push((path, item))
                }
                self.0.efvttcs.set(efvttcs)
                    .unwrap_or_else(|_| log::warn!(
                        "concurrent call to the same \
                        ExposureCtrl.pair_files_efvttcs()"
                    ));
                self.0.efvttcs.get()
                    .expect("efvttsc has just been set!")
            }
        })
    }

    pub async fn list_files_efvttcs(
        &'p self,
    ) -> Result<Vec<&'p EFViewTaskTemplatesCtrl<'p>>, PlatformError> {
        Ok(self.pair_files_efvttcs()
            .await?
            .iter()
            .filter_map(|(_, efvttc)| efvttc.as_ref())
            .collect::<Vec<_>>())
    }

    pub async fn pair_files_profile_prompt_groups(
        &'p self,
    ) -> Result<
        Vec<(&'p str, Option<(ExposureFileProfile, UserPromptGroupRefs<'p>)>)>,
        PlatformError
    > {
        let efvttsc = self.pair_files_efvttcs().await?;
        let mut result = Vec::new();
        for (path, value) in efvttsc.iter() {
            let item = if let Some(efvttsc) = value {
                Some((
                    efvttsc.exposure_file_ctrl().profile().await?,
                    efvttsc.create_user_prompt_groups()?,
                ))
            } else {
                None
            };
            result.push((path.as_str(), item))
        }
        Ok(result)
    }

    pub fn exposure(&self) -> &ExposureRef<'p> {
        &self.0.exposure
    }

}
