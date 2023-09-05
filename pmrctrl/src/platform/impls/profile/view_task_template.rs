use pmrcore::{
    profile::{
        ViewTaskTemplate,
        traits::ViewTaskTemplateBackend,
    },
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use crate::{
    error::PlatformError,
    handle::ExposureCtrl,
    platform::Platform,
};

impl<
    'a,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> Platform<'a, MCP, TMP> {
    pub async fn adds_view_task_template(
        &'a self,
        view_task_template: ViewTaskTemplate,
    ) -> Result<i64, PlatformError> {
        // TODO return the view task template id after going through both
        // the MC/TM platform?
        todo!()
    }

    pub async fn get_view_task_template(
        &'a self,
        id: i64,
    ) -> Result<(), PlatformError> {
        // actually formulate the question; likely used by the profile
        // one level up.
        todo!()
    }
}
