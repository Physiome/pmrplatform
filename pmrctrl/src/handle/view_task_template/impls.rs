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
};
use super::VTTCTask;

impl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> ViewTaskTemplatesCtrl<'p, 'db, MCP, TMP>
where
    'p: 'db
{
    pub(crate) fn new(
        platform: &'p Platform<'db, MCP, TMP>,
        exposure_file: ExposureFileRef<'db, MCP>,
        view_task_templates: ViewTaskTemplates,
    ) -> Self {
        Self {
            platform,
            exposure_file,
            // TODO figure out how to include and OnceLock the
            // exposure_file_ctrl,
            // ... only if it's meant to be one
            view_task_templates,
            choice_registry: OnceLock::new(),
            choice_registry_cache: OnceLock::new(),
        }
    }

    async fn get_registry(
        &self
    ) -> Result<&PreparedChoiceRegistry, PlatformError> {
        Ok(match self.choice_registry.get() {
            Some(registry) => Ok::<_, PlatformError>(registry),
            None => {
                let exposure = self.platform.get_exposure(
                    self.exposure_file.exposure_id()
                ).await?;
                self.choice_registry.set(
                    (&exposure).try_into()?
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
        &'p self
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
        &'p self,
    ) -> Result<Vec<UserArgRef>, PlatformError> {
        let cache = self.get_registry_cache().await?;
        Ok(UserArgBuilder::from((
            self.view_task_templates.as_slice(),
            cache,
        )).collect::<Vec<_>>())
    }

    /// This creates a mapping from the ViewTaskTemplates that are being
    /// controlled by this handle.  The mapping goes from each element's
    /// id to the task that it should be spawning.
    pub async fn create_tasks_from_input(
        &'p self,
        user_input: &'db UserInputMap,
    ) -> Result<Vec<VTTCTask>, PlatformError> {
        let cache = self.get_registry_cache().await?;

        let exposure = self.exposure_file.exposure().await?;

        // TODO figure out how to prepare this directory
        // TODO figure out how to get the source data into here
        //      maybe a trait for workspace?  the workspace checkout controller?
        let mut basedir = self.platform.data_root.clone();
        basedir.push("exposure");
        basedir.push(exposure.id().to_string());
        // the view identifier
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
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> From<&'p ViewTaskTemplatesCtrl<'db, 'db, MCP, TMP>> for &'p ViewTaskTemplates
where
    'p: 'db
{
    fn from(item: &'p ViewTaskTemplatesCtrl<'db, 'db, MCP, TMP>) -> Self {
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
