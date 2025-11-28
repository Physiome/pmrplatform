use leptos::{server, server_fn};
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

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/workspace_root_policy_state",
    responses((
        status = 200,
        description = "Check PolicyState for the current user for the /workspace/ endpoint.",
        body = PolicyState
    ), AppError),
))]
#[server(endpoint = "workspace_root_policy_state")]
pub async fn workspace_root_policy_state() -> Result<PolicyState, AppError> {
    Ok(session().await?
        .enforcer_and_policy_state("/workspace/", "").await?)
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/list_workspaces",
    responses((
        status = 200,
        description = "List of workspaces within an EnforcedOk",
        body = EnforcedOk<Workspaces>,
    ), AppError),
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
    ), AppError),
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

// this struct is a placeholder to help utoipa
#[cfg(feature = "utoipa")]
#[allow(dead_code)]
#[derive(utoipa::ToSchema)]
struct WorkspaceInfoArgs {
    id: Id,
    commit: Option<String>,
    path: Option<String>,
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/get_workspace_info",
    request_body(
        description = r#"
Get details associated with a workspace. Default values will be used where null is provided.

Default commit is the top level commit.

Default path points to the root of the repo.
        "#,
        content((
            WorkspaceInfoArgs = "application/json",
            examples(
                ("Example 1" = (
                    summary = "Using an alias to identify workspace to select its top level information.",
                    value = json!({
                        "commit": null,
                        "id": {
                          "Aliased": "beeler_reuter_1977"
                        },
                        "path": null
                    })
                )),
                ("Example 2" = (
                    summary = "Access details of a file within a commit using a number to identify workspace",
                    value = json!({
                        "commit": "cb090c96a2ce627457b14def4910ac39219b8340",
                        "id": {
                          "Number": "684"
                        },
                        "path": "beeler_reuter_1977.cellml"
                    })
                )),
            )
        )),
    ),
    responses((
        status = 200,
        description = "Get information related to a workspace",
        body = EnforcedOk<RepoResult>,
    ), AppError),
))]
#[server(
    input = server_fn::codec::Json,
    endpoint = "get_workspace_info",
)]
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

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/get_log_info",
    request_body(
        description = r#"
Get log entries associated with a workspace.
        "#,
        content((
            Id = "application/json",
            examples(
                ("Example 1" = (
                    summary = "Get log entries associated a specific workspace by alias.",
                    value = json!({
                        "id": {
                          "Aliased": "beeler_reuter_1977"
                        },
                    })
                )),
            )
        )),
    ),
    responses((
        status = 200,
        description = "Log entries from the workspace",
        body = EnforcedOk<RepoResult>,
    ), AppError),
))]
#[server(
    input = server_fn::codec::Json,
    endpoint = "get_log_info",
)]
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

// this struct is a placeholder to help utoipa
#[cfg(feature = "utoipa")]
#[allow(dead_code)]
#[derive(utoipa::ToSchema)]
struct CreateWorkspaceArgs {
    uri: String,
    description: Option<String>,
    long_description: Option<String>,
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/create_workspace",
    request_body(
        description = r#"
Create a workspace.
        "#,
        content((
            CreateWorkspaceArgs = "application/x-www-form-urlencoded",
        )),
    ),
    responses((
        status = 200,
        description = "Path to the new workspace.",
        body = String,
        example = "/workspace/123/",
    ), AppError),
))]
#[server(
    endpoint = "create_workspace",
)]
pub async fn create_workspace_core(
    uri: String,
    description: Option<String>,
    long_description: Option<String>,
) -> Result<String, AppError> {
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
    Ok(format!("/workspace/{}/", entry.alias))
}

#[server]
pub async fn create_workspace(
    uri: String,
    description: Option<String>,
    long_description: Option<String>,
) -> Result<(), AppError> {
    let location = create_workspace_core(uri, description, long_description).await?;
    leptos_axum::redirect(&location);
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
