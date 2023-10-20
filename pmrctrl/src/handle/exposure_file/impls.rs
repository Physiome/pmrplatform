use pmrcore::{
    exposure::{
        traits::{
            ExposureFile,
            ExposureFileViewBackend,
        },
    },
    platform::{
        MCPlatform,
        TMPlatform,
    },
};

use super::ExposureFileCtrl;
use crate::{
    error::PlatformError,
    handle::ExposureFileViewCtrl,
};

impl<
    'db,
    'repo,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> ExposureFileCtrl<'db, 'repo, MCP, TMP> {
    pub async fn create_view(
        &'db self,
        exposure_file_view_id: i64,
        view_task_template_id: i64,
    ) -> Result<ExposureFileViewCtrl<'db, MCP, TMP>, PlatformError> {
        // TODO write proper tests for this to verify the whole workflow between
        // all the related moving pieces.
        let efvb: &dyn ExposureFileViewBackend = &self.platform.mc_platform;
        let exposure_file_view = self
            .platform
            .mc_platform
            .get_exposure_file_view(
                efvb.insert(
                    self.exposure_file.id(),
                    view_task_template_id,
                    None,
                ).await?
            )
            .await?;
        Ok(ExposureFileViewCtrl {
            platform: self.platform,
            exposure_file_view,
        })
    }
}
