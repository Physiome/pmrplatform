use leptos::{
    prelude::ServerFnError,
    server,
};
use pmrcore::{
    repo::{
        LogInfo,
        RepoResult,
    },
    workspace::Workspaces,
};
use crate::{
    enforcement::EnforcedOk,
    error::AppError,
};

#[cfg(feature = "ssr")]
mod ssr {
    pub use pmrcore::{
        ac::{
            agent::Agent,
            role::Role,
            workflow::State,
        },
        workspace::traits::{
            Workspace,
            WorkspaceBackend,
        },
    };
    pub use crate::{
        ac::api::session,
        server::platform,
    };
}
#[cfg(feature = "ssr")]
use self::ssr::*;

#[server]
pub async fn list_workspaces() -> Result<EnforcedOk<Workspaces>, ServerFnError<AppError>> {
    let policy_state = session().await?
        .enforcer_and_policy_state("/workspace/", "").await?;
    let platform = platform().await?;
    Ok(policy_state.to_enforced_ok(
        WorkspaceBackend::list_workspaces(
            platform.mc_platform.as_ref()).await
            .map_err(|_| AppError::InternalServerError)?
    ))
}

#[server]
pub async fn get_workspace_info(
    id: i64,
    commit: Option<String>,
    path: Option<String>,
) -> Result<EnforcedOk<RepoResult>, ServerFnError<AppError>> {
    let policy_state = session().await?
        .enforcer_and_policy_state(format!("/workspace/{id}/"), "").await?;
    let platform = platform().await?;
    let handle = platform.repo_backend()
        .git_handle(id).await
        .map_err(|_| AppError::InternalServerError)?;
    match (commit.as_ref(), path.as_ref(), handle.repo()) {
        (None, None, Err(_)) => Ok(policy_state.to_enforced_ok(RepoResult {
            workspace: handle.workspace().clone_inner(),
            commit: None,
            path: None,
            target: None,
        })),
        (_, _, Err(_)) => Err(AppError::InternalServerError)?,
        (_, _, Ok(_)) => Ok(policy_state.to_enforced_ok(handle
            .pathinfo(commit, path)
            .map_err(|_| AppError::InternalServerError)?
            .into()
        )),
    }
}

#[server]
pub async fn get_log_info(
    id: i64,
) -> Result<EnforcedOk<LogInfo>, ServerFnError<AppError>> {
    let policy_state = session().await?
        .enforcer_and_policy_state(format!("/workspace/{id}/"), "").await?;
    let platform = platform().await?;
    let handle = platform.repo_backend()
        .git_handle(id).await
        .map_err(|_| AppError::InternalServerError)?;
    // FIXME only returning up to the first 30 log entries.
    Ok(policy_state.to_enforced_ok(handle.loginfo(None, None, Some(30))
        .map_err(|_| AppError::InternalServerError)?))
}

#[server]
pub async fn create_workspace(
    uri: String,
    description: String,
    long_description: String,
) -> Result<(), ServerFnError<AppError>> {
    let policy_state = session().await?
        .enforcer_and_policy_state("/workspace/", "create").await?;
    let platform = platform().await?;
    // First create the workspace
    let ctrl = platform.create_workspace(
        &uri,
        &description,
        &long_description,
    )
        .await
        .map_err(|_| AppError::InternalServerError)?;

    // then set the default workflow state to private
    let id = ctrl.workspace().id();
    let resource = format!("/workspace/{id}/");
    platform
        .ac_platform
        .set_wf_state_for_res(&resource, State::Private)
        .await
        .map_err(|_| AppError::InternalServerError)?;
    // and grant the current user the owner permission

    if let Some(policy) = policy_state.policy {
        if let Agent::User(user) = policy.agent {
            platform
                .ac_platform
                .res_grant_role_to_agent(&resource, user, Role::Owner)
                .await
                .map_err(|_| AppError::InternalServerError)?;
        }
    }

    leptos_axum::redirect(format!("/workspace/{id}").as_ref());
    Ok(())
}

#[server]
pub async fn synchronize(
    id: i64,
) -> Result<(), ServerFnError<AppError>> {
    session().await?
        .enforcer(format!("/workspace/{id}/"), "protocol_write").await?;
    let platform = platform().await?;
    platform.repo_backend()
        .sync_workspace(id).await
        .map_err(|_| AppError::InternalServerError)?;
    Ok(())
}
