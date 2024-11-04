use leptos::{
    prelude::ServerFnError,
    server,
};
use pmrcore::{
    repo::RepoResult,
    workspace::Workspaces,
};
use crate::error::AppError;

#[cfg(feature = "ssr")]
mod ssr {
    pub use pmrcore::workspace::traits::WorkspaceBackend;
    pub use crate::{
        ac::api::enforcer,
        server::platform,
    };
}
#[cfg(feature = "ssr")]
use self::ssr::*;

#[server]
pub async fn list_workspaces() -> Result<Workspaces, ServerFnError> {
    let platform = platform().await?;
    Ok(WorkspaceBackend::list_workspaces(platform.mc_platform.as_ref())
        .await?)
}

#[server]
pub async fn get_workspace_info(
    id: i64,
    commit: Option<String>,
    path: Option<String>,
) -> Result<RepoResult, ServerFnError<AppError>> {
    enforcer(format!("/workspace/{id}/"), "").await?;
    let platform = platform().await?;
    Ok(platform.repo_backend()
        .git_handle(id).await
        .map_err(|_| AppError::InternalServerError)?
        .pathinfo(commit, path)
        .map_err(|_| AppError::InternalServerError)?
        .into()
    )
}
