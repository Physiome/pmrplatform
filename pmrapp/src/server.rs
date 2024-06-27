use leptos::{
    ServerFnError,
};
use pmrcore::platform::{
    MCPlatform,
    TMPlatform,
};
use pmrctrl::platform::Platform;
use pmrmodel::backend::db::SqliteBackend;

pub async fn platform_context<MCP, TMP>() -> Result<Platform<MCP, TMP>, ServerFnError>
where
    MCP: MCPlatform + Clone + Sized + Send + Sync + 'static,
    TMP: TMPlatform + Clone + Sized + Send + Sync + 'static,
{
    Ok(leptos_axum::extract::<axum::Extension<Platform<MCP, TMP>>>()
        .await?
        .0
    )
}

// TODO figure out how the specific type can be avoided, the goal
// is to allow generics for the backend.
pub async fn platform() -> Result<Platform<SqliteBackend, SqliteBackend>, ServerFnError> {
    platform_context().await
}

pub mod workspace;
