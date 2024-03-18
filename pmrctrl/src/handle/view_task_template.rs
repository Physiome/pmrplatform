// TODO this module prbably should be a submodule to exposure?
// though given only the struct is exported this isn't too critical.
use pmrcore::{
    platform::{
        MCPlatform,
        TMPlatform,
    },
    profile::ViewTaskTemplates,
    task::Task,
};
use pmrmodel::registry::{
    PreparedChoiceRegistry,
    PreparedChoiceRegistryCache,
};
use std::sync::OnceLock;

use crate::{
    handle::ExposureFileCtrl,
    platform::Platform,
};

pub struct ViewTaskTemplatesCtrl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    platform: &'p Platform<'db, MCP, TMP>,
    exposure_file_ctrl: ExposureFileCtrl<'p, 'db, MCP, TMP>,
    view_task_templates: ViewTaskTemplates,
    choice_registry: OnceLock<PreparedChoiceRegistry>,
    choice_registry_cache: OnceLock<PreparedChoiceRegistryCache<'db>>,
}

/// These are for task that spawned off a ViewTaskTemplatesCtrl
pub struct VTTCTask {
    pub(crate) view_task_template_id: i64,
    pub(crate) task: Task,
}

mod impls;
