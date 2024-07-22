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
    MCP: MCPlatform + Sized + Send + Sync,
    TMP: TMPlatform + Sized + Send + Sync,
> {
    pub(crate) platform: &'p Platform<MCP, TMP>,
    // TODO there needs to be an Arc<ExposureFileCtrl> stored here
    pub exposure_file_view: ExposureFileViewRef<'p>,
}

mod impls;
