use leptos::{
    ServerFnError,
    server,
};
use pmrcore::{
    workspace::Workspaces,
    platform::{
        MCPlatform,
        TMPlatform,
    },
};

#[cfg(feature = "ssr")]
pub mod ssr {
    use leptos::use_context;
    use pmrcore::workspace::traits::WorkspaceBackend;
    use pmrctrl::platform::Platform;
    use pmrmodel::backend::db::SqliteBackend;
    use super::*;

    pub fn platform<MCP, TMP>() -> Result<Platform<MCP, TMP>, ServerFnError>
    where
        MCP: MCPlatform + Clone + Sized + Send + Sync + 'static,
        TMP: TMPlatform + Clone + Sized + Send + Sync + 'static,
    {
        use_context::<Platform<MCP, TMP>>()
            .ok_or_else(|| ServerFnError::ServerError("Missing Platform.".into()))
    }

    pub async fn list_workspaces() -> Result<Workspaces, ServerFnError> {
        // TODO figure out how the specific type can be avoided.
        let platform = platform::<SqliteBackend, SqliteBackend>()?;
        Ok(WorkspaceBackend::list_workspaces(platform.mc_platform.as_ref())
            .await?)
    }
}

#[server]
pub async fn list_workspaces() -> Result<Workspaces, ServerFnError> {
    ssr::list_workspaces().await
}
