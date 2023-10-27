// TODO this module prbably should be a submodule to exposure?
// though given only the struct is exported this isn't too critical.
use pmrcore::{
    exposure::ExposureFileRef,
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

use crate::platform::Platform;

pub struct ViewTaskTemplatesCtrl<
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    platform: &'db Platform<'db, MCP, TMP>,
    exposure_file: ExposureFileRef<'db, MCP>,
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
