use pmrcore::{
    exposure::{
        task::traits::ExposureTaskBackend,
        traits::ExposureFileView as _,
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

impl<'p> ExposureFileViewCtrl<'p> {
    /// Queue a Task created by ViewTaskTemplateCtrl
    ///
    /// This consumes the incoming task.
    ///
    /// Returns a tuple containing the newly created ExposureFileView.id
    /// and the Task.id
    pub async fn queue_task(
        &mut self,
        vttc_task: VTTCTask,
    ) -> Result<(i64, i64), PlatformError> {
        // The reason why this consumes the incoming item is because the
        // task is basically provided not in a state that was already in
        // the db, the underlying API depands it, and dropping the data
        // to be queued is a way to prevent duplicating this call.
        let (vtt_id, task): (i64, Task) = vttc_task.into();
        let tmp = self.platform.tm_platform.as_ref();
        let task = TaskBackend::adds_task(tmp, task).await?;
        let mcp = self.platform.mc_platform.as_ref();
        let efv_id = ExposureTaskBackend::create_task_for_view(
            mcp,
            self.exposure_file_view.id(),
            vtt_id,
            Some(task.id),
        ).await?;
        self.exposure_file_view
            .update_exposure_file_view_task_id(Some(efv_id))
            .await?;
        Ok((efv_id, task.id))
    }
}
