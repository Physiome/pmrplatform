use pmrcore::{
    exposure::traits::{
        Exposure,
        ExposureBackend,
    },
    workspace::traits::Workspace as _,
};

use crate::{
    error::PlatformError,
    handle::ExposureCtrl,
    platform::Platform,
};

impl<'p> Platform {
    /// Creates an exposure with all the relevant data validated.
    ///
    /// Returns a `ExposureCtrl` handle.
    pub async fn create_exposure(
        &self,
        workspace_id: i64,
        commit_id: &str,
    ) -> Result<ExposureCtrl, PlatformError> {
        let git_handle = self
            .repo_backend()
            .git_handle(workspace_id).await?;

        // // This verifies the existence of the commit
        // // FIXME this has borrow lifetime issues still when called in here
        // {
        //     let _ = git_handle.pathinfo(Some(commit_id), None)?;
        // }

        // This verifies the existence of the commit
        let _ = git_handle.check_commit(commit_id)?;

        // workspace_id and commit verified, create the root exposure
        let exposure = self.mc_platform.get_exposure(
            ExposureBackend::insert(
                self.mc_platform.as_ref(),
                git_handle.workspace().description(),
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
    ) -> Result<ExposureCtrl<'p>, PlatformError> {
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

// FIXME the following methods are here to workaround lifetime issues.
// I really should rewrite the entire backend to be backed by a common arena.
impl Platform {
    /// With the user input applied to all files within this exposure, process them
    /// all at once.  This is analogous to the exposure build step.
    ///
    /// Returns the number of tasks queued.
    pub async fn process_vttc_tasks_for_exposure(&self, id: i64) -> Result<usize, PlatformError> {
        let exposure = self.get_exposure(id).await?;
        let mut args = Vec::new();
        for efvttc in exposure.list_files_efvttcs().await? {
            let profile = efvttc
                .exposure_file_ctrl()
                .profile()
                .await?;
            args.push((efvttc, profile));
        }

        let mut result = 0;
        for (efvttc, profile) in args.iter() {
            if let Some(profile) = profile {
                let vttc_tasks = efvttc.create_tasks_from_input(&profile.user_input)?;
                result += efvttc
                    .exposure_file_ctrl()
                    .process_vttc_tasks(vttc_tasks).await?
                    .len();
            }
        }
        Ok(result)
    }

}
