use pmrcore::{
    exposure::ExposureFileRef,
    platform::{
        MCPlatform,
        TMPlatform,
    },
    profile::ViewTaskTemplates,
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

mod impls;
