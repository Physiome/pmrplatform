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
    // TODO need to figure out if reference to underlying repo is needed
    pub exposure_file_view: ExposureFileViewRef<'db, MCP>,
}

mod impls;
