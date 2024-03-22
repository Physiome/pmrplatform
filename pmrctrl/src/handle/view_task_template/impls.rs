use pmrcore::{
    exposure::{
        traits::ExposureFile as _,
        task::ExposureFileViewTaskTemplate,
    },
    task::Task,
    platform::{
        MCPlatform,
        TMPlatform,
    },
    profile::{
        ViewTaskTemplate,
        ViewTaskTemplates,
    },
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
        PreparedChoiceRegistry,
        PreparedChoiceRegistryCache,
    },
};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        OnceLock,
    },
};

use crate::{
    error::PlatformError,
    handle::{
        ExposureFileCtrl,
        ViewTaskTemplatesCtrl,
        view_task_template::EFViewTaskTemplateCtrl,
    },
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
        exposure_file_ctrl: ExposureFileCtrl<'p, 'db, MCP, TMP>,
        view_task_templates: ViewTaskTemplates,
    ) -> Self {
        Self {
            platform,
            exposure_file_ctrl,
            view_task_templates,
            choice_registry: OnceLock::new(),
            choice_registry_cache: OnceLock::new(),
            efvttcs: OnceLock::new(),
        }
    }

    async fn get_registry(
        &self
    ) -> Result<&PreparedChoiceRegistry, PlatformError> {
        Ok(match self.choice_registry.get() {
            Some(registry) => Ok::<_, PlatformError>(registry),
            None => {
                self.choice_registry.set(Arc::new(
                    (&self.exposure_file_ctrl).try_into()?
                )).unwrap_or_else(|_| log::warn!(
                    "concurrent call to the same \
                    ViewTaskTemplateCtrls.registry_cache()"
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
                self.choice_registry_cache.set(Arc::new(
                    (registry as &dyn ChoiceRegistry<_>).into()
                )).unwrap_or_else(|_| log::warn!(
                    "concurrent call to the same \
                    ViewTaskTemplateCtrls.choice_registry_cache()"
                ));
                Ok(self.choice_registry_cache.get()
                    .expect("choice_registry_cache just been set!"))
            }
        }?)
    }

    fn get_efvttcs(
        &'p self
    ) -> &'p [EFViewTaskTemplateCtrl<MCP, TMP>] {
        self.efvttcs.get_or_init(|| self.view_task_templates
            .iter()
            .map(|efvtt| {
                let mut reg_basedir = PreparedChoiceRegistry::new();
                reg_basedir.register("working_dir", HashMap::from([
                    ("working_dir".to_string(), Some(
                        self.exposure_file_ctrl
                            .data_root()
                            .join(efvtt.view_key.clone())
                            .as_path()
                            .display()
                            .to_string()
                    )),
                ]).into());
                let registry = vec![
                    // need direct access to the Arc
                    self.choice_registry.get()
                        .expect("this should have been already set")
                        .clone(),
                    Arc::new(reg_basedir),
                ];
                let ctrl = EFViewTaskTemplateCtrl::new(
                    self.platform,
                    self.exposure_file_ctrl.clone(),
                    efvtt,
                    registry
                );
                ctrl
            })
            .collect::<Vec<_>>()
        )
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
        user_input: &'p UserInputMap,
    ) -> Result<Vec<VTTCTask>, PlatformError> {
        // prepare the registry
        let _ = self.get_registry_cache().await?;
        let basedir = self.exposure_file_ctrl.data_root();

        // TODO how to store these ctrls temporarily but long enough to get
        // the tasks returned?
        let tasks = self.get_efvttcs().into_iter()
            .map(|ctrl| ctrl.create_task_from_input(&user_input))
            .collect::<Result<Vec<_>, BuildArgErrors>>()?;

        // let tasks = self
        //     .view_task_templates
        //     .iter()
        //     .map(|efvtt| {
        //         let view_basedir = basedir.join(&efvtt.view_key).as_path().display().to_string();
        //         let mut reg_basedir = PreparedChoiceRegistry::new();
        //         reg_basedir.register("working_dir", HashMap::from([
        //             ("working_dir".to_string(), Some(view_basedir.clone())),
        //         ]).into());
        //         let reg_basedir_cache = ChoiceRegistryCache::from(
        //             &reg_basedir as &dyn ChoiceRegistry<_>,
        //         );
        //         let view_cache = ChoiceRegistryCache::from(&[
        //             &reg_basedir_cache,
        //             cache,
        //         ]);
        //         let mut task = Task::from(TaskBuilder::try_from((
        //             user_input,
        //             efvtt.task_template
        //                 .as_ref()
        //                 .expect("task_template must have been provided"),
        //             // &view_cache,
        //             cache,
        //         ))?);
        //         // TODO how to actually inject this basedir as `working_dir`
        //         // to the lookup cache above?  Maybe provide a layer on top?
        //         task.basedir = view_basedir;
        //         Ok(VTTCTask {
        //             view_task_template_id: efvtt.id,
        //             task: task,
        //         })
        //     })
        //     .collect::<Result<Vec<_>, BuildArgErrors>>()?;

        Ok(tasks)
    }
}

impl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> EFViewTaskTemplateCtrl<'p, 'db, MCP, TMP>
where
    'p: 'db
{
    pub(crate) fn new(
        platform: &'p Platform<'db, MCP, TMP>,
        exposure_file_ctrl: ExposureFileCtrl<'p, 'db, MCP, TMP>,
        efvtt: &'db ViewTaskTemplate,
        choice_registry: Vec<Arc<PreparedChoiceRegistry>>,
    ) -> Self {
        Self {
            platform,
            exposure_file_ctrl,
            efvtt,
            choice_registry: choice_registry,
            choice_registry_cache: OnceLock::new(),
        }
    }

    fn get_registry_cache(
        &'db self
    ) -> &PreparedChoiceRegistryCache {
        match self.choice_registry_cache.get() {
            Some(registry_cache) => registry_cache,
            None => {
                self.choice_registry_cache.set(
                    self.choice_registry
                        .as_slice()
                        .into_iter()
                        .map(|x| x.as_ref() as &dyn ChoiceRegistry<_>)
                        .collect::<Vec<_>>()
                        .into()
                ).unwrap_or_else(|_| log::warn!(
                    "concurrent call to the same \
                    EFViewTaskTemplateCtrl.choice_registry_cache()"
                ));
                self.choice_registry_cache.get()
                    .expect("choice_registry_cache just been set!")
            }
        }
    }

    fn create_task_from_input(
        &'p self,
        user_input: &'db UserInputMap,
    ) -> Result<VTTCTask, BuildArgErrors> {
          // TODO resolve this value from the registry
          let view_basedir = self.exposure_file_ctrl
              .data_root()
              .join(&self.efvtt.view_key)
              .as_path()
              .display()
              .to_string();
          let cache = self.get_registry_cache();
          let mut task = Task::from(TaskBuilder::try_from((
              user_input,
              self.efvtt.task_template
                  .as_ref()
                  .expect("task_template must have been provided"),
              cache,
          ))?);
          task.basedir = view_basedir;
          Ok(VTTCTask {
              view_task_template_id: self.efvtt.id,
              task: task,
          })
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
