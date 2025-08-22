use pmrcore::{
    exposure::{
        ExposureFileViewRef,
        task::traits::ExposureTaskBackend,
        traits::{
            ExposureFile as _,
            ExposureFileView as _,
        },
    },
    task::{
        Task,
        traits::TaskBackend,
    },
};
use std::{
    fmt,
    path::PathBuf,
};

use super::ExposureFileViewCtrl;
use crate::{
    error::{
        CtrlError,
        PlatformError,
    },
    handle::{
        ExposureFileCtrl,
        view_task_template::VTTCTask,
    },
    platform::Platform,
};

impl fmt::Debug for ExposureFileViewCtrl<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExposureFileViewCtrl<'_>")
            .field("platform", &self.platform)
            .field("exposure_file.id", &self.exposure_file.exposure_file().id())
            .field("exposure_file_view.id", &self.exposure_file_view.id())
            .field("exposure_file_view.view_key", &self.exposure_file_view.view_key())
            .field("view_path", &self.view_path)
            .finish()
    }
}

impl<'p> ExposureFileViewCtrl<'p> {
    pub(crate) fn new<S>(
        platform: &'p Platform,
        exposure_file_view: ExposureFileViewRef<'p>,
        exposure_file: ExposureFileCtrl<'p>,
        view_path: Option<S>,
    ) -> Self
    where
        S: Into<String>
    {
        Self {
            platform,
            exposure_file_view,
            exposure_file,
            view_path: view_path.map(Into::into),
        }
    }

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

    pub async fn read_blob(&self, path: &str) -> Result<Vec<u8>, CtrlError> {
        let view_key = self.exposure_file_view
            .view_key()
            .ok_or(CtrlError::EFVCIncomplete)?;
        self.exposure_file.read_blob(view_key, path).await
    }

    pub fn exposure_file_ctrl(&self) -> &ExposureFileCtrl<'p> {
        &self.exposure_file
    }

    pub fn exposure_file_view(&self) -> &ExposureFileViewRef<'p> {
        &self.exposure_file_view
    }

    pub fn view_key(&self) -> Option<&str> {
        self.exposure_file_view.view_key()
    }

    pub fn view_path(&self) -> Option<&str> {
        self.view_path.as_deref()
    }

    pub fn data_root(&self) -> Result<PathBuf, CtrlError> {
        self.exposure_file_view
            .view_key()
            .map(|view_key| {
                let mut target = PathBuf::from(self.exposure_file_ctrl().data_root());
                target.push(view_key);
                target
            })
            .ok_or(CtrlError::EFVCIncomplete)
    }

    pub fn working_dir(&self) -> Result<PathBuf, CtrlError> {
        let mut result = self.data_root()?;
        result.push("work");
        Ok(result)
    }
}
