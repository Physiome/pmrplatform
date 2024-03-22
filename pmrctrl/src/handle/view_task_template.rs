// TODO this module prbably should be a submodule to exposure?
// though given only the struct is exported this isn't too critical.
use pmrcore::{
    exposure::task::ExposureFileViewTaskTemplate,
    platform::{
        MCPlatform,
        TMPlatform,
    },
    profile::{
        ViewTaskTemplate,
        ViewTaskTemplates,
    },
    task::Task,
};
use pmrmodel::registry::{
    PreparedChoiceRegistry,
    PreparedChoiceRegistryCache,
};
use std::sync::{
    Arc,
    OnceLock,
};

use crate::{
    handle::ExposureFileCtrl,
    platform::Platform,
};

/// A controller for exposure view task templates for the given
// TODO rename this to EFViewTaskTemplatesCtrl
pub struct ViewTaskTemplatesCtrl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    platform: &'p Platform<'db, MCP, TMP>,
    exposure_file_ctrl: ExposureFileCtrl<'p, 'db, MCP, TMP>,
    view_task_templates: ViewTaskTemplates,
    choice_registry: OnceLock<Arc<PreparedChoiceRegistry>>,
    choice_registry_cache: OnceLock<Arc<PreparedChoiceRegistryCache<'db>>>,
    efvttcs: OnceLock<Vec<EFViewTaskTemplateCtrl<'p, 'db, MCP, TMP>>>,
}

// This crate only struct is an attempt to work around a lifetime issue
// when trying to merge multiple choice registries together.
pub(crate) struct EFViewTaskTemplateCtrl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    platform: &'p Platform<'db, MCP, TMP>,
    exposure_file_ctrl: ExposureFileCtrl<'p, 'db, MCP, TMP>,
    efvtt: &'db ViewTaskTemplate,
    choice_registry: Vec<Arc<PreparedChoiceRegistry>>,
    choice_registry_cache: OnceLock<PreparedChoiceRegistryCache<'db>>,
}

/// These are for task that spawned off a ViewTaskTemplatesCtrl
pub struct VTTCTask {
    pub(crate) view_task_template_id: i64,
    pub(crate) task: Task,
}

mod impls;
