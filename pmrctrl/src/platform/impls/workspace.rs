use pmrrepo::handle::GitHandle;

use crate::{
    error::PlatformError,
    handle::WorkspaceCtrl,
    platform::Platform,
};

impl<'p> Platform {
    pub async fn get_workspace(
        &'p self,
        id: i64,
    ) -> Result<WorkspaceCtrl<'p>, PlatformError> {
        let git_handle = self.repo_backend.git_handle(id).await?;
        let platform = self;
        Ok(ExposureCtrl::new(
            platform,
            git_handle,
        ))
    }
}
