use leptos::server;
use pmrcore::{
    alias::AliasEntry,
    repo::{
        LogInfo,
        RepoResult,
    },
    workspace::Workspace,
};
use crate::{
    app::id::Id,
    enforcement::{EnforcedOk, PolicyState},
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
            Workspace as _,
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

pub type Workspaces = Vec<AliasEntry<Workspace>>;

#[server(endpoint = "workspace_root_policy_state")]
pub async fn workspace_root_policy_state() -> Result<PolicyState, AppError> {
    Ok(session().await?
        .enforcer_and_policy_state("/workspace/", "").await?)
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/list_workspaces",
    responses(
        (status = 200, description = "List of workspaces within an EnforcedOk", body = EnforcedOk<Workspaces>),
    ),
))]
#[server(endpoint = "list_workspaces")]
pub async fn list_workspaces() -> Result<EnforcedOk<Workspaces>, AppError> {
    let policy_state = session().await?
        .enforcer_and_policy_state("/workspace/", "").await?;
    let platform = platform().await?;
    Ok(policy_state.to_enforced_ok(
        WorkspaceBackend::list_workspaces(platform.mc_platform.as_ref()).await
            .map_err(|_| AppError::InternalServerError)?
            .into_iter()
            .map(|workspace| AliasEntry {
                alias: workspace.id.to_string(),
                entity: workspace,
            })
            .collect()
    ))
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/list_aliased_workspaces",
    responses((
        status = 200,
        description = "List of workspaces with their alias within an EnforcedOk",
        body = EnforcedOk<Workspaces>,
    )),
))]
#[server(endpoint = "list_aliased_workspaces")]
pub async fn list_aliased_workspaces() -> Result<EnforcedOk<Workspaces>, AppError> {
    let policy_state = session().await?
        .enforcer_and_policy_state("/workspace/", "").await?;
    let platform = platform().await?;
    let workspaces = platform.mc_platform.list_aliased_workspaces()
        .await
        .map_err(|_| AppError::InternalServerError)?
        .into_iter()
        .map(|workspace| workspace.map(|entity| entity.into_inner()))
        .collect();
    Ok(policy_state.to_enforced_ok(workspaces))
}

// TODO move this to server::workspace instead?
#[cfg(feature = "ssr")]
pub(crate) async fn resolve_id(id: Id) -> Result<i64, AppError> {
    Ok(match id {
        Id::Number(s) => s.parse().map_err(|_| AppError::NotFound)?,
        Id::Aliased(s) => platform()
            .await?
            .mc_platform
            .resolve_alias("workspace", &s)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or(AppError::NotFound)?,
    })
}

#[server]
pub async fn get_workspace_info(
    // TODO this may need to be a enum to disambiguate the id vs. alias
    id: Id,
    commit: Option<String>,
    path: Option<String>,
) -> Result<EnforcedOk<RepoResult>, AppError> {
    let id = resolve_id(id).await?;
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
    id: Id,
) -> Result<EnforcedOk<LogInfo>, AppError> {
    let id = resolve_id(id).await?;
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
    description: Option<String>,
    long_description: Option<String>,
) -> Result<(), AppError> {
    let policy_state = session().await?
        .enforcer_and_policy_state("/workspace/", "create").await?;
    let platform = platform().await?;
    // First create the workspace
    let entry = platform.mc_platform.create_aliased_workspace(
        &uri,
        description.as_deref(),
        long_description.as_deref(),
    )
        .await
        .map_err(|_| AppError::InternalServerError)?;

    // then set the default workflow state to private
    let id = entry.entity.id();
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

    leptos_axum::redirect(format!("/workspace/{}", entry.alias).as_ref());
    Ok(())
}

#[server]
pub async fn synchronize(
    id: i64,
) -> Result<(), AppError> {
    session().await?
        .enforcer(format!("/workspace/{id}/"), "protocol_write").await?;
    let platform = platform().await?;
    platform.repo_backend()
        .sync_workspace(id).await
        .map_err(|_| AppError::InternalServerError)?;
    Ok(())
}
