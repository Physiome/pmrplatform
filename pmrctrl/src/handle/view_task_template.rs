use pmrcore::{
    exposure::ExposureFileRef,
    platform::{
        MCPlatform,
        TMPlatform,
    },
    profile::ViewTaskTemplates,
};

use crate::platform::Platform;

pub struct ViewTaskTemplatesCtrl<
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    pub(crate) platform: &'db Platform<'db, MCP, TMP>,
    pub(crate) exposure_file: ExposureFileRef<'db, MCP>,
    pub(crate) view_task_templates: ViewTaskTemplates,
}

mod impls;
