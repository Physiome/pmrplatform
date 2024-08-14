use futures::future;
use pmrcore::{
    exposure::{
        task::traits::ExposureTaskTemplateBackend,
        traits::{
            Exposure as _,
            ExposureFile as _,
            ExposureFileViewBackend,
        },
        ExposureFileRef,
    },
    task_template::traits::TaskTemplateBackend
};
use pmrrepo::handle::GitHandleResult;
use std::{
    fmt,
    path::PathBuf,
    sync::Arc,
};

use crate::{
    error::{
        CtrlError,
        PlatformError,
    },
    handle::{
        ExposureCtrl,
        ExposureFileCtrl,
        ExposureFileViewCtrl,
        EFViewTaskTemplatesCtrl,
        exposure_file::RawExposureFileCtrl,
        view_task_template::VTTCTask,
    },
    platform::Platform,
};

impl Clone for ExposureFileCtrl<'_> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl fmt::Debug for ExposureFileCtrl<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExposureFileCtrl<'_>")
            .field("platform", &self.0.platform)
            .field("exposure_id", &self.0.exposure.exposure().id())
            .field("exposure_file_id", &self.0.exposure_file.id())
            .finish()
    }
}

impl<'p> ExposureFileCtrl<'p> {
    pub fn new(
        platform: &'p Platform,
        exposure: ExposureCtrl<'p>,
        exposure_file: ExposureFileRef<'p>,
        pathinfo: GitHandleResult<'p>,
    ) -> Self {
        Self(Arc::new(RawExposureFileCtrl {
            platform,
            exposure,
            exposure_file,
            pathinfo,
        }))
    }

    /// Create a view from template
    ///
    /// Returns an ExposureFileViewCtrl for the view that just got
    /// created.
    pub async fn create_view_from_template(
        &self,
        view_task_template_id: i64,
    ) -> Result<ExposureFileViewCtrl<'p>, PlatformError> {
        self.get_view(
            ExposureFileViewBackend::insert(
                self.0.platform.mc_platform.as_ref(),
                self.exposure_file().id(),
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
    ) -> Result<ExposureFileViewCtrl<'p>, PlatformError> {
        // TODO write proper tests for this to verify the whole workflow between
        // all the related moving pieces.
        let exposure_file_view = self.0
            .platform
            .mc_platform
            .get_exposure_file_view(exposure_file_view_id)
            .await?;
        Ok(ExposureFileViewCtrl::new(
            self.0.platform,
            exposure_file_view,
            self.clone(),
            None::<String>,
        ))
    }

    /// Returns an ExposureFileViewCtrl for an existing view by the
    /// provided viewstr, which is a str that is `/` separated and
    /// the first segment is parsed as the view_key
    pub async fn resolve_view_by_viewstr(
        &self,
        viewstr: &str,
    ) -> Result<ExposureFileViewCtrl<'p>, CtrlError> {
        let mut splitter = viewstr.splitn(2, '/');
        let view_key = splitter.next().expect("must have first part");
        let view_path = splitter.next();
        let exposure_file_view = self.0
            .platform
            .mc_platform
            .get_exposure_file_view_by_file_view_key(
                self.exposure_file().id(),
                view_key,
            )
            .await
            // Assumes all DB errors to be this issue.
            .map_err(|_| CtrlError::EFVCNotFound(view_key.to_string()))?;
        Ok(ExposureFileViewCtrl::new(
            self.0.platform,
            exposure_file_view,
            self.clone(),
            view_path,
        ))
    }

    /// Ensure a view from a view task template id
    ///
    /// Returns an ExposureFileViewCtrl for the view that just got
    /// created.
    pub async fn ensure_view_from_template(
        &self,
        view_task_template_id: i64,
    ) -> Result<ExposureFileViewCtrl<'p>, PlatformError> {
        let exposure_file_view = self.0.platform
            .mc_platform
            .get_exposure_file_view_by_file_template(
                self.exposure_file()
                    .id(),
                ExposureFileViewBackend::insert(
                    self.0.platform.mc_platform.as_ref(),
                    self.exposure_file().id(),
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
        Ok(ExposureFileViewCtrl::new(
            self.0.platform,
            exposure_file_view,
            self.clone(),
            None::<String>,
        ))
    }

    /// Build a EFViewTaskTemplatesCtrl.
    ///
    /// This could be an impl on async TryFrom.
    ///
    /// Note that this would freeze the view templates associated with
    /// this particular instance of ExposureFileCtrl.
    pub async fn build_vttc(
        &'p self,
    ) -> Result<EFViewTaskTemplatesCtrl<'p>, PlatformError> {
        let mut vtts = ExposureTaskTemplateBackend::get_file_templates(
            self.0.platform.mc_platform.as_ref(),
            self.exposure_file().id(),
        ).await?;
        future::try_join_all(vtts.iter_mut().map(|vtt| async {
            Ok::<(), PlatformError>(vtt.task_template = Some(
                TaskTemplateBackend::get_task_template_by_id(
                    self.0.platform.tm_platform.as_ref(),
                    vtt.task_template_id,
                ).await?
            ))
        })).await?;
        Ok(EFViewTaskTemplatesCtrl::new(
            self.clone(),
            vtts.into(),
        ))
    }

    /// Process tasks produced via `ViewTaskTemplatesCtrl.create_tasks`
    /// into views
    pub async fn process_vttc_tasks(
        &self,
        vttc_tasks: Vec<VTTCTask>,
    ) -> Result<Vec<(i64, i64)>, PlatformError> {
        let mut iter = vttc_tasks.into_iter();
        let mut results: Vec<(i64, i64)> = Vec::new();
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

    pub fn pathinfo(&self) -> &GitHandleResult<'p> {
        &self.0.pathinfo
    }

    pub fn exposure_file(&self) -> &ExposureFileRef<'p> {
        &self.0.exposure_file
    }

    pub fn data_root(&self) -> PathBuf {
        let mut data_root = self.0.platform.data_root.join("exposure");
        data_root.push(self.0.exposure.exposure().id().to_string());
        data_root.push(self.0.exposure_file.id().to_string());
        data_root
    }
}
