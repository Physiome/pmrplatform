use leptos::{
    ServerFnError,
    server,
};
use pmrcore::{
    repo::RepoResult,
    workspace::{
        Workspaces,
        traits::WorkspaceBackend,
    },
    platform::{
        MCPlatform,
        TMPlatform,
    },
};
#[cfg(feature = "ssr")]
use crate::server::platform;

#[server]
pub async fn list_workspaces() -> Result<Workspaces, ServerFnError> {
    let platform = platform()?;
    Ok(WorkspaceBackend::list_workspaces(platform.mc_platform.as_ref())
        .await?)
}

#[server]
pub async fn get_workspace_info(id: i64) -> Result<RepoResult, ServerFnError> {
    let platform = platform()?;
    Ok(platform.repo_backend()
        .git_handle(id).await?
        .pathinfo::<String>(None, None)?
        .into()
    )
}
