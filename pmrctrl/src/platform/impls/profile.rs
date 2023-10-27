use pmrcore::{
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
use crate::{
    error::PlatformError,
    platform::Platform,
};

impl<
    'a,
    MCP: MCPlatform + Sized + Sync,
    TMP: TMPlatform + Sized + Sync,
> Platform<'a, MCP, TMP> {
    pub async fn create_view_profile(
        &self,
    ) -> Result<(), PlatformError> {
        todo!()
    }

    pub async fn get_profile(
        &self,
        _id: i64,
    ) -> Result<(), PlatformError> {
        // TODO assuming this will return the templates associated with this profile
        // also figure out how to formulate the question
        todo!()
    }
}

mod view_task_template;
