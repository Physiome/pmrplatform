use pmrcore::{
    exposure::ExposureFileViewRef,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};

use crate::platform::Platform;

pub struct ExposureFileViewCtrl<
    'p,
    'db,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> {
    pub(crate) platform: &'p Platform<'db, MCP, TMP>,
    // TODO there needs to be an Arc<ExposureFileCtrl> stored here
    pub exposure_file_view: ExposureFileViewRef<'db, MCP>,
}

mod impls;
