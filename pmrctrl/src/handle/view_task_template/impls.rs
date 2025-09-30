use pmrcore::{
    exposure::traits::ExposureFile as _,
    profile::{
        ViewTaskTemplate,
        ViewTaskTemplates,
    },
    task::Task,
    task_template::{
        TaskTemplateArg,
        UserInputMap,
    }
};
use pmrmodel::{
    error::BuildArgErrors,
    model::{
        profile::UserPromptGroupRefs,
        task_template::{
            TaskArgBuilder,
            TaskBuilder,
            UserArgBuilder,
            UserArgRefs,
        },
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
    error::{
        CtrlError,
        PlatformError,
    },
    handle::{
        ExposureFileCtrl,
        EFViewTaskTemplatesCtrl,
        view_task_template::EFViewTaskTemplateCtrl,
    },
};
use super::VTTCTask;

impl<'p> EFViewTaskTemplatesCtrl<'p> {
    pub(crate) fn new(
        exposure_file_ctrl: ExposureFileCtrl<'p>,
        view_task_templates: ViewTaskTemplates,
    ) -> Self {
        Self {
            exposure_file_ctrl,
            view_task_templates,
            choice_registry: OnceLock::new(),
            choice_registry_cache: OnceLock::new(),
            efvttcs: OnceLock::new(),
            task_template_args: OnceLock::new(),
        }
    }

    fn get_registry(
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

    pub fn get_registry_cache(
        &'p self
    ) -> Result<PreparedChoiceRegistryCache<'p>, PlatformError> {
        Ok(match self.choice_registry_cache.get() {
            Some(registry_cache) => Ok::<_, PlatformError>(registry_cache),
            None => {
                let registry = self.get_registry()?;
                self.choice_registry_cache.set(
                    (registry as &dyn ChoiceRegistry<_>).into()
                ).unwrap_or_else(|_| log::warn!(
                    "concurrent call to the same \
                    ViewTaskTemplateCtrls.choice_registry_cache()"
                ));
                Ok(self.choice_registry_cache.get()
                    .expect("choice_registry_cache just been set!"))
            }
        }?.clone())
    }

    fn get_efvttcs(
        &'p self
    ) -> &'p [EFViewTaskTemplateCtrl<'p>] {
        self.efvttcs.get_or_init(|| self.view_task_templates
            .iter()
            .map(|efvtt| {
                // FIXME should this be prepared at `get_registry`?
                // normally get_registry should provide everything, but
                // currently this working_dir registry is generally not
                // provided for end-user consumption (as in, users that
                // provide raw values they want to validate against),
                // and so if this situation changes (e.g. additional
                // local registries needed) this will need to be done
                // by then.
                let mut reg_basedir = PreparedChoiceRegistry::new();
                reg_basedir.register("working_dir", HashMap::from([
                    ("working_dir".to_string(), Some(
                        self.exposure_file_ctrl
                            .data_root()
                            .join(efvtt.view_key.clone())
                            .join("work")
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
                    self.exposure_file_ctrl.clone(),
                    efvtt,
                    registry
                );
                ctrl
            })
            .collect::<Vec<_>>()
        )
    }

    /// Get a TaskTemplateArg that may form one of the arguments in any
    /// of the TaskTemplates that are tracked internal to this control
    /// by the task_template_arg.id
    pub fn get_arg(
        &'p self,
        id: &i64,
    ) -> Option<&'p TaskTemplateArg> {
        self.task_template_args
            .get_or_init(|| self.view_task_templates
                .iter()
                .map(
                    |vtt| vtt.task_template
                        .as_ref()
                        .expect("task_template should have been included here")
                        .args
                        .as_ref()
                        .expect("args should have been included here")
                        .iter()
                        .map(|a| (a.id, a))
                )
                .flatten()
                .collect::<HashMap<_, _>>()
            )
            .get(id)
            .copied()
    }

    /// This provides a flat list of user args.
    pub fn create_user_arg_refs(
        &'p self,
    ) -> Result<UserArgRefs<'p>, PlatformError> {
        let cache = self.get_registry_cache()?;
        // shouldn't this be made to work?
        // Ok((&self.view_task_templates, cache).into())
        Ok(
            UserArgBuilder::from((
                self.view_task_templates.as_slice(),
                cache,
            ))
                .collect::<Vec<_>>()
                .into()
        )
    }

    /// This provides user args grouped by the prompt sets, which may be
    /// better for end user consumption.
    pub fn create_user_prompt_groups(
        &'p self,
    ) -> Result<UserPromptGroupRefs<'p>, PlatformError> {
        let cache = self.get_registry_cache()?;
        Ok((&self.view_task_templates, cache).into())
    }

    /// This creates a mapping from the ViewTaskTemplates that are being
    /// controlled by this handle.  The mapping goes from each element's
    /// id to the task that it should be spawning.
    pub fn create_tasks_from_input(
        &'p self,
        user_input: &'p UserInputMap,
    ) -> Result<Vec<VTTCTask>, PlatformError> {
        let _ = self.get_registry_cache()?;
        let tasks = self.get_efvttcs().into_iter()
            .map(|ctrl| ctrl.create_task_from_input(&user_input))
            .collect::<Result<Vec<_>, BuildArgErrors>>()?;
        Ok(tasks)
    }

    pub async fn update_user_input(
        &'p self,
        user_input: &'p UserInputMap,
    ) -> Result<(), PlatformError> {
        let mut checked_user_input = UserInputMap::new();

        for (arg_id, answer) in user_input.iter() {
            let arg = self.get_arg(&arg_id)
                .ok_or(CtrlError::ArgIdNotInProfile(*arg_id))?;

            match TaskArgBuilder::try_from((
                Some(answer.as_ref()),
                arg,
                self.get_registry_cache()?,
            )) {
                Ok(_) => checked_user_input.insert(*arg_id, answer.to_string()),
                Err(_) => None,
            };
        }

        let id = self.exposure_file_ctrl
            .exposure_file()
            .id();
        self.exposure_file_ctrl.0
            .platform
            .mc_platform
            .update_ef_user_input(
                id,
                &checked_user_input,
            ).await?;

        Ok(())
    }

    pub fn exposure_file_ctrl(&'p self) -> ExposureFileCtrl<'p> {
        self.exposure_file_ctrl.clone()
    }
}

impl<'p> EFViewTaskTemplateCtrl<'p> {
    pub(crate) fn new(
        exposure_file_ctrl: ExposureFileCtrl<'p>,
        efvtt: &'p ViewTaskTemplate,
        choice_registry: Vec<Arc<PreparedChoiceRegistry>>,
    ) -> Self {
        Self {
            exposure_file_ctrl,
            efvtt,
            choice_registry: choice_registry,
            choice_registry_cache: OnceLock::new(),
        }
    }

    fn get_registry_cache(
        &'p self
    ) -> PreparedChoiceRegistryCache<'p> {
        match self.choice_registry_cache.get() {
            Some(registry_cache) => registry_cache.clone(),
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
                    .clone()
            }
        }
    }

    fn create_task_from_input(
        &'p self,
        user_input: &'p UserInputMap,
    ) -> Result<VTTCTask, BuildArgErrors> {
          let cache = self.get_registry_cache();
          let mut task = Task::from(TaskBuilder::try_from((
              user_input,
              self.efvtt.task_template
                  .as_ref()
                  .expect("task_template must have been provided"),
              cache,
          ))?);
          // for now we just re-calculate this value rather than the
          // lookup because no idea which way is faster (nor does it
          // really matter at this point)
          task.basedir = self.exposure_file_ctrl
              .data_root()
              .join(&self.efvtt.view_key)
              .as_path()
              .display()
              .to_string();
          // if doing direct lookup from the registry
          // lookup from cache cannot yet be done.
          // task.basedir = self.choice_registry[1]
          //     .as_ref()
          //     .lookup("working_dir")
          //     .expect("registered in registry")
          //     .get("working_dir")
          //     .expect("value registered")
          //     .expect("not None")
          //     .to_string();
          Ok(VTTCTask {
              view_task_template_id: self.efvtt.id,
              task: task,
          })
    }
}

impl<'p> From<&'p EFViewTaskTemplatesCtrl<'p>>
for
    &'p ViewTaskTemplates
{
    fn from(item: &'p EFViewTaskTemplatesCtrl<'p>) -> Self {
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
