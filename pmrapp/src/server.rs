use leptos::{
    ServerFnError,
    use_context,
};
use pmrcore::platform::{
    MCPlatform,
    TMPlatform,
};
use pmrctrl::platform::Platform;
use pmrmodel::backend::db::SqliteBackend;

pub fn platform_context<MCP, TMP>() -> Result<Platform<MCP, TMP>, ServerFnError>
where
    MCP: MCPlatform + Clone + Sized + Send + Sync + 'static,
    TMP: TMPlatform + Clone + Sized + Send + Sync + 'static,
{
    use_context::<Platform<MCP, TMP>>()
        .ok_or_else(|| ServerFnError::ServerError("Missing Platform.".into()))
}

// TODO figure out how the specific type can be avoided, the goal
// is to allow generics for the backend.
pub fn platform() -> Result<Platform<SqliteBackend, SqliteBackend>, ServerFnError> {
    platform_context()
}
