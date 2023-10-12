use futures::future;
use pmrcore::{
    exposure::task::traits::ExposureTaskTemplateBackend,
    platform::{
        MCPlatform,
        TMPlatform,
    },
    profile::{
        ViewTaskTemplates,
        traits::ViewTaskTemplateBackend,
    },
    task::Task,
    task_template::traits::TaskTemplateBackend
};
use pmrmodel::{
    error::BuildArgErrors,
    model::task_template::UserInputMap,
    registry::ChoiceRegistryCache,
};
use crate::{
    error::PlatformError,
    handle::{
        ExposureCtrl,
        ViewTaskTemplatesCtrl,
    },
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
    ) -> Result<ViewTaskTemplatesCtrl<'db, MCP, TMP>, PlatformError> {
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
        Ok(ViewTaskTemplatesCtrl {
            platform: &self,
            exposure_file,
            view_task_templates: vtts.into(),
        })
    }

    pub async fn create_tasks<T>(
        &'db self,
        view_task_templates: &ViewTaskTemplates,
        user_input: &UserInputMap,
        cache: ChoiceRegistryCache<'_, T>
    ) -> Result<Vec<Task>, BuildArgErrors> {
        // While I originally thought the ViewTaskTemplates could provide
        // an impl for doing this, it unfortunately sits at the pmrcore while
        // the types it need to bind to are at pmrmodel which lies downstream,
        // so the types wouldn't be available in pmrcore and the impl can't
        // be done in pmrmodel due to orphan rule, but really what's needed
        // for now is the ability to put the task into the database, so we
        // are just going to do that here right away.
        // TODO figure out if a task handle should be provided - it will
        // provide a means to inspect the task and some fn commit to bring
        // the final task into the database.
        todo!();
    }
}
