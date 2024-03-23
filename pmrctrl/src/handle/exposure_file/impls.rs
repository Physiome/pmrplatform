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
    platform::{
        MCPlatform,
        TMPlatform,
    },
    task_template::traits::TaskTemplateBackend
};
use pmrrepo::handle::GitHandleResult;
use std::{
    path::Path,
    sync::Arc,
};

use crate::{
    error::PlatformError,
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

impl<
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
> Clone for ExposureFileCtrl<'_, MCP, TMP> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<
    'p,
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
> ExposureFileCtrl<'p, MCP, TMP> {
    pub fn new(
        platform: &'p Platform<MCP, TMP>,
        exposure: ExposureCtrl<'p, MCP, TMP>,
        exposure_file: ExposureFileRef<'p, MCP>,
        pathinfo: GitHandleResult<'p, MCP>,
    ) -> Self {
        let mut data_root = platform.data_root.join("exposure");
        data_root.push(exposure.exposure().id().to_string());
        data_root.push(exposure_file.id().to_string());
        Self(Arc::new(RawExposureFileCtrl {
            platform,
            exposure,
            exposure_file,
            pathinfo,
            data_root,
        }))
    }

    /// Create a view from template
    ///
    /// Returns an ExposureFileViewCtrl for the view that just got
    /// created.
    pub async fn create_view_from_template(
        &self,
        view_task_template_id: i64,
    ) -> Result<ExposureFileViewCtrl<'p, MCP, TMP>, PlatformError> {
        let efvb: &dyn ExposureFileViewBackend = self.0.platform.mc_platform.as_ref();
        self.get_view(
            efvb.insert(
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
    ) -> Result<ExposureFileViewCtrl<'p, MCP, TMP>, PlatformError> {
        // TODO write proper tests for this to verify the whole workflow between
        // all the related moving pieces.
        let exposure_file_view = self.0
            .platform
            .mc_platform
            .get_exposure_file_view(exposure_file_view_id)
            .await?;
        Ok(ExposureFileViewCtrl {
            platform: self.0.platform,
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
    ) -> Result<ExposureFileViewCtrl<'p, MCP, TMP>, PlatformError> {
        let efvb: &dyn ExposureFileViewBackend = self.0.platform.mc_platform.as_ref();
        let exposure_file_view = self.0.platform
            .mc_platform
            .get_exposure_file_view_by_file_template(
                self.exposure_file()
                    .id(),
                efvb
                    .insert(
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
        Ok(ExposureFileViewCtrl {
            platform: self.0.platform,
            exposure_file_view,
        })
    }

    /// Build a EFViewTaskTemplatesCtrl.
    ///
    /// This could be an impl on async TryFrom.
    ///
    /// Note that this would freeze the view templates associated with
    /// this particular instance of ExposureFileCtrl.
    pub async fn build_vttc(
        &'p self,
    ) -> Result<EFViewTaskTemplatesCtrl<'p, MCP, TMP>, PlatformError> {
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
            &self.0.platform,
            self.clone(),
            vtts.into(),
        ))
    }

    /// Process tasks produced via `ViewTaskTemplatesCtrl.create_tasks`
    /// into views
    pub async fn process_vttc_tasks(
        &self,
        vttc_tasks: Vec<VTTCTask>,
    ) -> Result<Vec<i64>, PlatformError> {
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

    pub fn pathinfo(&self) -> &GitHandleResult<'p, MCP> {
        &self.0.pathinfo
    }

    pub fn exposure_file(&self) -> &ExposureFileRef<'p, MCP> {
        &self.0.exposure_file
    }

    pub fn data_root(&self) -> &Path {
        self.0.data_root.as_ref()
    }
}
