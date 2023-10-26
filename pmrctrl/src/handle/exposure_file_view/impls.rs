use pmrcore::{
    exposure::{
        ExposureFileView,
        task::traits::ExposureTaskBackend,
        traits::{
            ExposureFileView as _,
            ExposureFileViewBackend,
        },
    },
    platform::{
        MCPlatform,
        TMPlatform,
    },
    task::{
        Task,
        traits::TaskBackend,
    },
};

use super::ExposureFileViewCtrl;
use crate::{
    error::PlatformError,
    handle::view_task_template::VTTCTask,
};

impl<
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> ExposureFileViewCtrl<'db, MCP, TMP> {
    /// Queue a Task created by ViewTaskTemplateCtrl
    ///
    /// This consumes the incoming task.
    pub async fn queue_task(
        &mut self,
        vttc_task: VTTCTask,
    ) -> Result<i64, PlatformError> {
        // The reason why this consumes the incoming item is because the
        // task is basically provided not in a state that was already in
        // the db, the underlying API depands it, and dropping the data
        // to be queued is a way to prevent duplicating this call.
        let (vtt_id, task): (i64, Task) = vttc_task.into();
        let tb: &dyn TaskBackend = &self.platform.tm_platform;
        let task = tb.adds_task(task).await?;
        let etb: &dyn ExposureTaskBackend = &self.platform.mc_platform;
        let efv_id = etb.create_task_for_view(
            self.exposure_file_view.id(),
            vtt_id,
            Some(task.id),
        ).await?;
        self.exposure_file_view
            .update_exposure_file_view_task_id(Some(efv_id))
            .await?;
        Ok(efv_id)
    }
}
