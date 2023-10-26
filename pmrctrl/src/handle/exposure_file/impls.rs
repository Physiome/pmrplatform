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
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> ExposureFileCtrl<'db, MCP, TMP> {
    /// Create a view from template
    ///
    /// Returns an ExposureFileViewCtrl for the view that just got
    /// created.
    pub async fn create_view_from_template(
        &self,
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
        &self,
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

    /// Ensure a view from a view task template id
    ///
    /// Returns an ExposureFileViewCtrl for the view that just got
    /// created.
    pub async fn ensure_view_from_template(
        &self,
        view_task_template_id: i64,
    ) -> Result<ExposureFileViewCtrl<'db, MCP, TMP>, PlatformError> {
        let efvb: &dyn ExposureFileViewBackend = &self.platform.mc_platform;
        let exposure_file_view = self.platform
            .mc_platform
            .get_exposure_file_view_by_file_template(
                self.exposure_file
                    .id(),
                efvb
                    .insert(
                        self.exposure_file.id(),
                        view_task_template_id,
                        None,
                    )
                    .await
                    // ultimately the view_task_template_id is used for
                    // the query
                    .map(|_| view_task_template_id)
                    .unwrap_or(view_task_template_id),
            )
            .await?;
        Ok(ExposureFileViewCtrl {
            platform: self.platform,
            exposure_file_view,
        })
    }

    /// Process tasks produced via `ViewTaskTemplateCtrl.create_tasks`
    /// into views
    pub async fn process_vttc_tasks(
        &self,
        vttc_tasks: Vec<VTTCTask>,
    ) -> Result<Vec<i64>, PlatformError> {
        let exposure_file_id = self.exposure_file.id();
        let mut iter = vttc_tasks.into_iter();
        let mut results: Vec<i64> = Vec::new();
        // TODO determine if benefits of sequential insertion is
        // actually required here.
        while let Some(vttc_task) = iter.next() {
            let mut efv_ctrl = self.ensure_view_from_template(
                vttc_task.view_task_template_id
            ).await?;
            results.push(efv_ctrl.queue_task(vttc_task).await?);
        }
        Ok(results)
    }
}
