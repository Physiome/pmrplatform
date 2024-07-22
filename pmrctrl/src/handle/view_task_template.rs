// TODO this module prbably should be a submodule to exposure?
// though given only the struct is exported this isn't too critical.
use pmrcore::{
    profile::{
        ViewTaskTemplate,
        ViewTaskTemplates,
    },
    task::Task,
    task_template::TaskTemplateArg,
};
use pmrmodel::registry::{
    PreparedChoiceRegistry,
    PreparedChoiceRegistryCache,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        OnceLock,
    },
};

use crate::handle::ExposureFileCtrl;

/// A controller for exposure view task templates for the given
pub struct EFViewTaskTemplatesCtrl<'p> {
    exposure_file_ctrl: ExposureFileCtrl<'p>,
    view_task_templates: ViewTaskTemplates,
    choice_registry: OnceLock<Arc<PreparedChoiceRegistry>>,
    choice_registry_cache: OnceLock<Arc<PreparedChoiceRegistryCache<'p>>>,
    efvttcs: OnceLock<Vec<EFViewTaskTemplateCtrl<'p>>>,
    task_template_args: OnceLock<HashMap<i64, &'p TaskTemplateArg>>,
}

/// Individual controller for each of the view_task_template of the above.
pub(crate) struct EFViewTaskTemplateCtrl<'p> {
    exposure_file_ctrl: ExposureFileCtrl<'p>,
    efvtt: &'p ViewTaskTemplate,
    choice_registry: Vec<Arc<PreparedChoiceRegistry>>,
    choice_registry_cache: OnceLock<PreparedChoiceRegistryCache<'p>>,
}

/// These are for task that spawned off a EFViewTaskTemplatesCtrl
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct VTTCTask {
    pub(crate) view_task_template_id: i64,
    pub(crate) task: Task,
}

mod impls;
