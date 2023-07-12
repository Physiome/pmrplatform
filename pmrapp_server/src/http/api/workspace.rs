use axum::{
    extract::{Extension, Path},
    Json,
    routing::get,
    Router,
};
use pmrrepo::git::{
    WorkspaceGitResult,
    PmrBackendWR,
};
use pmrmodel_base::{
    merged::WorkspacePathInfo,
    workspace::{
        Workspaces,
        traits::WorkspaceBackend,
    }
};
use std::path::PathBuf;

use pmrapp_client::model::JsonWorkspaceRecord;
use crate::http::AppContext;
use crate::http::{Error, Result};


pub fn router() -> Router {
    Router::new()
        .route("/", get(api_workspace))
        .route("/:workspace_id/",
            get(api_workspace_top))
        .route("/:workspace_id/file/",
            get(api_workspace_pathinfo_workspace_id))
        .route("/:workspace_id/file/:commit_id/",
            get(api_workspace_pathinfo_workspace_id_commit_id))
        .route("/:workspace_id/file/:commit_id/*path",
            get(api_workspace_pathinfo_workspace_id_commit_id_path))
        .route("/:workspace_id/raw/:commit_id/*path",
            get(api_workspace_pathinfo_workspace_id_commit_id_path))
}

pub async fn api_workspace(ctx: Extension<AppContext>) -> Result<Json<Workspaces>> {
    let workspaces = WorkspaceBackend::list_workspaces(&ctx.backend).await?;
    Ok(Json(workspaces))
}

pub async fn api_workspace_top(
    ctx: Extension<AppContext>,
    Path(workspace_id): Path<i64>,
) -> Result<Json<JsonWorkspaceRecord>> {
    let workspace = match WorkspaceBackend::get_workspace_by_id(&ctx.backend, workspace_id).await {
        Ok(workspace) => workspace,
        Err(_) => return Err(Error::NotFound),
    };
    let commit_id = match PmrBackendWR::new(
        &ctx.backend,
        PathBuf::from(&ctx.config.pmr_git_root),
        &workspace
    ) {
        Ok(pmrbackend) => match pmrbackend.pathinfo(None, None) {
            Ok(result) => Some(format!("{}", result.commit.id())),
            Err(_) => None,
        },
        Err(_) => None
    };
    Ok(Json(JsonWorkspaceRecord {
        workspace: workspace,
        head_commit: commit_id,
    }))
}

pub async fn api_workspace_top_ssr(
    ctx: Extension<AppContext>,
    Path(workspace_id): Path<i64>,
) -> Result<(JsonWorkspaceRecord, Option<WorkspacePathInfo>)> {
    let workspace = match WorkspaceBackend::get_workspace_by_id(&ctx.backend, workspace_id).await {
        Ok(workspace) => workspace,
        Err(_) => return Err(Error::NotFound),
    };
    let pmrbackend = PmrBackendWR::new(
        &ctx.backend,
        PathBuf::from(&ctx.config.pmr_git_root),
        &workspace
    )?;

    let (head_commit, path_info) = match pmrbackend.pathinfo(None, None) {
        Ok(result) => (
            Some(format!("{}", result.commit.id())),
            Some(<WorkspacePathInfo>::from(
                &WorkspaceGitResult::new(
                    &pmrbackend.workspace,
                    &result,
                )
            )),
        ),
        Err(_) => (None, None)
    };
    Ok((
        JsonWorkspaceRecord {
            workspace: workspace,
            head_commit: head_commit,
        },
        path_info,
    ))
}

async fn api_workspace_pathinfo(
    ctx: Extension<AppContext>,
    workspace_id: i64,
    commit_id: Option<String>,
    path: Option<String>,
) -> Result<Json<WorkspacePathInfo>> {
    let workspace = match WorkspaceBackend::get_workspace_by_id(&ctx.backend, workspace_id).await {
        Ok(workspace) => workspace,
        Err(_) => return Err(Error::NotFound),
    };
    let pmrbackend = PmrBackendWR::new(
        &ctx.backend,
        PathBuf::from(&ctx.config.pmr_git_root),
        &workspace
    )?;

    let result = match pmrbackend.pathinfo(
        commit_id.as_deref(),
        path.as_deref(),
    ) {
        Ok(result) => Ok(Json(<WorkspacePathInfo>::from(
            &WorkspaceGitResult::new(
                &workspace,
                &result,
            )
        ))),
        Err(e) => {
            // TODO log the URI triggering these messages?
            log::info!("pmrbackend.pathinfo error: {:?}", e);
            Err(Error::NotFound)
        }
    };
    result
}

pub async fn api_workspace_pathinfo_workspace_id(
    ctx: Extension<AppContext>,
    Path(workspace_id): Path<i64>,
) -> Result<Json<WorkspacePathInfo>> {
    api_workspace_pathinfo(ctx, workspace_id, None, None).await
}

pub async fn api_workspace_pathinfo_workspace_id_commit_id(
    ctx: Extension<AppContext>,
    Path((workspace_id, commit_id)): Path<(i64, String)>,
) -> Result<Json<WorkspacePathInfo>> {
    api_workspace_pathinfo(ctx, workspace_id, Some(commit_id), None).await
}


pub async fn api_workspace_pathinfo_workspace_id_commit_id_path(
    ctx: Extension<AppContext>,
    Path((workspace_id, commit_id, path)): Path<(i64, String, String)>,
) -> Result<Json<WorkspacePathInfo>> {
    api_workspace_pathinfo(ctx, workspace_id, Some(commit_id), Some(path)).await
}

