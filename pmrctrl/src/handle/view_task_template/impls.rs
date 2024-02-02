use pmrcore::{
    exposure::{
        ExposureFileRef,
        traits::{
            Exposure,
            ExposureFile,
        },
    },
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
        PreparedChoiceRegistry,
        PreparedChoiceRegistryCache,
    },
};
use std::{
    path::PathBuf,
    sync::OnceLock,
};

use crate::{
    error::PlatformError,
    handle::ViewTaskTemplatesCtrl,
    platform::Platform,
    registry::make_choice_registry,
};
use super::VTTCTask;

impl<
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> ViewTaskTemplatesCtrl<'db, MCP, TMP> {
    pub(crate) fn new(
        platform: &'db Platform<'db, MCP, TMP>,
        exposure_file: ExposureFileRef<'db, MCP>,
        view_task_templates: ViewTaskTemplates,
    ) -> Self {
        Self {
            platform,
            exposure_file,
            view_task_templates,
            choice_registry: OnceLock::new(),
            choice_registry_cache: OnceLock::new(),
        }
    }

    async fn get_registry(
        &'db self
    ) -> Result<&PreparedChoiceRegistry, PlatformError> {
        Ok(match self.choice_registry.get() {
            Some(registry) => Ok::<_, PlatformError>(registry),
            None => {
                let exposure = self.platform.get_exposure(
                    self.exposure_file.exposure_id()
                ).await?;
                self.choice_registry.set(
                    make_choice_registry(&exposure)?
                ).unwrap_or_else(|_| log::warn!(
                    "concurrent call to the same \
                    ViewTaskTemplateCtrl.registry_cache()"
                ));
                Ok(self.choice_registry.get()
                    .expect("choice_registry just been set!"))
            }
        }?)
    }

    async fn get_registry_cache(
        &'db self
    ) -> Result<&PreparedChoiceRegistryCache, PlatformError> {
        Ok(match self.choice_registry_cache.get() {
            Some(registry_cache) => Ok::<_, PlatformError>(registry_cache),
            None => {
                let registry = self.get_registry().await?;
                self.choice_registry_cache.set(
                    ChoiceRegistryCache::from(registry as &dyn ChoiceRegistry<_>),
                ).unwrap_or_else(|_| log::warn!(
                    "concurrent call to the same \
                    ViewTaskTemplateCtrl.choice_registry_cache()"
                ));
                Ok(self.choice_registry_cache.get()
                    .expect("choice_registry_cache just been set!"))
            }
        }?)
    }

    pub async fn create_user_arg_refs(
        &'db self,
    ) -> Result<Vec<UserArgRef>, PlatformError> {
        let cache = self.get_registry_cache().await?;
        Ok(UserArgBuilder::from((
            self.view_task_templates.as_slice(),
            cache,
        )).collect::<Vec<_>>())
    }

    /// This creates a mapping from the ViewTaskTemplates that are being
    /// controlled by this handle.  The mapping goes from each element's
    /// id to the task that it should be spawnning.
    pub async fn create_tasks_from_input(
        &'db self,
        user_input: &'db UserInputMap,
    ) -> Result<Vec<VTTCTask>, PlatformError> {
        let cache = self.get_registry_cache().await?;

        let exposure = self.exposure_file.exposure().await?;

        let mut basedir = self.platform.data_root.clone();
        basedir.push("exposure");
        basedir.push(exposure.id().to_string());
        basedir.push(self.exposure_file.id().to_string());

        let tasks = self
            .view_task_templates
            .iter()
            .map(|efvtt| {
                let mut task = Task::from(TaskBuilder::try_from((
                    user_input,
                    efvtt.task_template
                        .as_ref()
                        .expect("task_template must have been provided"),
                    cache,
                ))?);
                // this would be the output dir
                // TODO need to figure out how to communicate the workspace
                // extraction
                // TODO probably at the start a VTTCTasks could be created
                // such that it contains a reference to the git archive.
                task.basedir = basedir.as_path().display().to_string();
                Ok(VTTCTask {
                    view_task_template_id: efvtt.id,
                    task: task,
                })
            })
            .collect::<Result<Vec<_>, BuildArgErrors>>()?;

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

impl From<VTTCTask> for (i64, Task) {
    fn from(item: VTTCTask) -> Self {
        (
            item.view_task_template_id,
            item.task,
        )
    }
}
