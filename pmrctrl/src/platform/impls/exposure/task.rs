use futures::future;
use pmrcore::{
    exposure::task::traits::ExposureTaskTemplateBackend,
    platform::{
        MCPlatform,
        TMPlatform,
    },
    task_template::traits::TaskTemplateBackend
};
use crate::{
    error::PlatformError,
    handle::ViewTaskTemplatesCtrl,
    platform::Platform,
};

impl<
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> Platform<'db, MCP, TMP> {
    /// Query for complete ViewTaskTemplates associated with the given
    /// exposure file.
    pub async fn get_file_templates_for_exposure_file(
        &'db self,
        exposure_file_id: i64,
    ) -> Result<ViewTaskTemplatesCtrl<'_, MCP, TMP>, PlatformError> {
        let mut vtts = ExposureTaskTemplateBackend::get_file_templates(
            &self.mc_platform,
            exposure_file_id,
        ).await?;
        future::try_join_all(vtts.iter_mut().map(|vtt| async {
            Ok::<(), PlatformError>(vtt.task_template = Some(
                TaskTemplateBackend::get_task_template_by_id(
                    &self.tm_platform,
                    vtt.task_template_id,
                ).await?
            ))
        })).await?;
        let exposure_file = self.mc_platform.get_exposure_file(
            exposure_file_id).await?;
        Ok(ViewTaskTemplatesCtrl::new(
            &self,
            exposure_file,
            vtts.into(),
        ))
    }
}
