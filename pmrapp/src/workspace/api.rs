use leptos::{
    ServerFnError,
    server,
};
use pmrcore::{
    repo::RepoResult,
    workspace::Workspaces,
};
#[cfg(feature = "ssr")]
use crate::server::platform;

#[server]
pub async fn list_workspaces() -> Result<Workspaces, ServerFnError> {
    use pmrcore::workspace::traits::WorkspaceBackend;

    let platform = platform().await?;
    Ok(WorkspaceBackend::list_workspaces(platform.mc_platform.as_ref())
        .await?)
}

#[server]
pub async fn get_workspace_info(
    id: i64,
    commit: Option<String>,
    path: Option<String>,
) -> Result<RepoResult, ServerFnError> {
    let platform = platform().await?;
    Ok(platform.repo_backend()
        .git_handle(id).await?
        .pathinfo(commit, path)?
        .into()
    )
}
