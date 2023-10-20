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
    handle::{
        ExposureFileViewCtrl,
        view_task_template::VTTCTask,
    },
};

impl<
    'db,
    'repo,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> ExposureFileCtrl<'db, 'repo, MCP, TMP> {
    /// Create a view from template
    ///
    /// Returns an ExposureFileViewCtrl for the view that just got
    /// created.
    pub async fn create_view_from_template(
        &'db self,
        view_task_template_id: i64,
    ) -> Result<ExposureFileViewCtrl<'db, MCP, TMP>, PlatformError> {
        let efvb: &dyn ExposureFileViewBackend = &self.platform.mc_platform;
        self.get_view(
            efvb.insert(
                self.exposure_file.id(),
                view_task_template_id,
                None,
            ).await?
        ).await
    }

    /// Returns an ExposureFileViewCtrl for an existing view by the id
    /// of the interested view.
    pub async fn get_view(
        &'db self,
        exposure_file_view_id: i64,
    ) -> Result<ExposureFileViewCtrl<'db, MCP, TMP>, PlatformError> {
        // TODO write proper tests for this to verify the whole workflow between
        // all the related moving pieces.
        let exposure_file_view = self
            .platform
            .mc_platform
            .get_exposure_file_view(exposure_file_view_id)
            .await?;
        Ok(ExposureFileViewCtrl {
            platform: self.platform,
            exposure_file_view,
        })
    }

    /// Process tasks produced via `ViewTaskTemplateCtrl.create_tasks`
    /// into views
    pub async fn process_vttc_tasks(
        &'db self,
        vttid_tasks: &[VTTCTask],
    ) -> Result<Vec<i64>, PlatformError> {
        todo!()
    }
}
