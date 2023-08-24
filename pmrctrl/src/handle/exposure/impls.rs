use pmrcore::{
    exposure::traits::ExposureFileBackend,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};

use super::ExposureCtrl;
use crate::{
    error::PlatformError,
};

impl<
    'a,
    MCP: MCPlatform + Sync,
    TMP: TMPlatform + Sync,
> ExposureCtrl<'a, MCP, TMP> {
    pub async fn create_file(
        &self,
        workspace_file_path: &str,
    ) -> Result<i64, PlatformError> {
        // quick failing here.
        let _ = self.git_handle.pathinfo(
            Some(&self.inner.commit_id),
            Some(workspace_file_path),
        )?;
        // path exists, so create the exposure file
        let efb: &dyn ExposureFileBackend = &self.platform.mc_platform;
        Ok(efb.insert(
            self.inner.id,
            workspace_file_path,
            None,
        ).await?)
    }
}
