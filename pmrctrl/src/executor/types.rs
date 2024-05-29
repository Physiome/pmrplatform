use pmrcore::platform::{
    MCPlatform,
    TMPlatform,
};
use crate::platform::Platform;

#[derive(Clone)]
pub struct Executor<
    MCP: MCPlatform + Clone + Send + Sync,
    TMP: TMPlatform + Clone + Send + Sync,
> {
    pub(crate) platform: Platform<MCP, TMP>,
}
