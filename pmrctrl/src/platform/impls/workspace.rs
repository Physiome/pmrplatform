use crate::{
    error::PlatformError,
    handle::WorkspaceCtrl,
    platform::Platform,
};

impl<'p> Platform {
    pub async fn create_workspace(
        &'p self,
        url: &str,
        description: Option<&str>,
        long_description: Option<&str>,
    ) -> Result<WorkspaceCtrl<'p>, PlatformError> {
        self.get_workspace(
            self.mc_platform.add_workspace(
                url,
                description,
                long_description,
            ).await?
        ).await
    }

    pub async fn get_workspace(
        &'p self,
        id: i64,
    ) -> Result<WorkspaceCtrl<'p>, PlatformError> {
        let workspace = self.mc_platform.get_workspace(id).await?;
        Ok(WorkspaceCtrl::new(
            self,
            workspace
        ))
    }
}
