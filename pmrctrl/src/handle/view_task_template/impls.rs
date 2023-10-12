use async_trait::async_trait;
use pmrcore::{
    exposure::traits::ExposureFile,
    task::Task,
    platform::{
        MCPlatform,
        TMPlatform,
    },
    profile::ViewTaskTemplates,
};
use pmrmodel::{
    error::BuildArgErrors,
    model::task_template::{
        TaskBuilder,
        UserArgBuilder,
        UserArgRef,
        UserInputMap,
    },
    registry::{
        ChoiceRegistry,
        ChoiceRegistryCache,
    },
};

use crate::{
    error::PlatformError,
    handle::ViewTaskTemplatesCtrl,
    registry::make_choice_registry,
};

impl<
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> ViewTaskTemplatesCtrl<'db, MCP, TMP> {

    // TODO make impl From?
    // TODO re-evaluate the referenced version because we will need to store
    // the registry as a OnceLock or something like that here...
    // ... which may actually be fine?
    /*
    pub async fn create_user_args(
        &'db self,
    ) -> Result<Vec<UserArgRef>, PlatformError> {
        let exposure = self.platform.get_exposure(
            self.exposure_file.exposure_id()
        ).await?;
        let registry = make_choice_registry(&exposure)?;
        let cache = ChoiceRegistryCache::from(&registry as &dyn ChoiceRegistry<_>);
        Ok(UserArgBuilder::from((
            self.view_task_templates.as_slice(),
            &cache,
        )).collect::<Vec<_>>())
    }
    */

    pub async fn create_tasks_from_input(
        &'db self,
        user_input: &UserInputMap,
    ) -> Result<Vec<Task>, PlatformError> {
        let exposure = self.platform.get_exposure(
            self.exposure_file.exposure_id()
        ).await?;
        let registry = make_choice_registry(&exposure)?;
        let cache = ChoiceRegistryCache::from(&registry as &dyn ChoiceRegistry<_>);

        let tasks = self
            .view_task_templates
            .iter()
            .map(|efvtt| Ok(Task::from(TaskBuilder::try_from((
                user_input,
                efvtt.task_template
                    .as_ref()
                    .expect("task_template must have been provided"),
                &cache,
            ))?)))
            .collect::<Result<Vec<_>, BuildArgErrors>>()?;

        // TODO figure out consequence of doing insertion directly here
        // without the intermediate step - maybe provide this method,
        // plus an insertion method (elsewhere) and another one here
        // that will insert the whole mess?
        //
        // there is a TaskCtrl that will be implemented, so how much
        // this relate to that needs to be figured out (is that going
        // to provide the insertion or something else?)
        //
        // for now just return this
        Ok(tasks)
    }

}

impl<
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> From<&'db ViewTaskTemplatesCtrl<'db, MCP, TMP>> for &'db ViewTaskTemplates {
    fn from(item: &'db ViewTaskTemplatesCtrl<'db, MCP, TMP>) -> Self {
        &item.view_task_templates
    }
}
