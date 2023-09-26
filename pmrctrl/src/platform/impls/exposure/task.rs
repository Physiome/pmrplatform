use futures::future;
use pmrcore::{
    exposure::task::traits::ExposureTaskBackend,
    platform::{
        MCPlatform,
        TMPlatform,
    },
    profile::{
        ViewTaskTemplate,
        traits::ViewTaskTemplateBackend,
    },
    task_template::traits::TaskTemplateBackend
};
use crate::{
    error::PlatformError,
    handle::ExposureCtrl,
    platform::Platform,
};

impl<
    'a,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> Platform<'a, MCP, TMP> {
    /// Query for complete ViewTaskTemplates associated with the given
    /// exposure file.
    pub async fn get_file_templates_for_exposure_file(
        &'a self,
        exposure_file_id: i64,
    ) -> Result<Vec<ViewTaskTemplate>, PlatformError> {
        let mut vtts = ExposureTaskBackend::get_file_templates(
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
        Ok(vtts)
    }
}

