use axum::{
    extract::{Extension, Path},
    Json,
    routing::get,
    Router,
};
use pmrrepo::backend::Backend;
use pmrcore::{
    repo::RepoResult,
    workspace::{
        Workspaces,
        traits::WorkspaceBackend,
    }
};

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
    let workspaces = WorkspaceBackend::list_workspaces(&ctx.db).await?;
    Ok(Json(workspaces))
}

pub async fn api_workspace_top(
    ctx: Extension<AppContext>,
    Path(workspace_id): Path<i64>,
) -> Result<Json<RepoResult>> {
    let backend = Backend::new(&ctx.db, (&ctx.config.pmr_git_root).into());
    let handle = backend.git_handle(workspace_id).await?;
    let pathinfo = handle.pathinfo(None, None)?;
    Ok(Json(pathinfo.into()))
}

/*
pub async fn api_workspace_top_ssr(
    ctx: Extension<AppContext>,
    Path(workspace_id): Path<i64>,
) -> Result<(JsonWorkspaceRecord, Option<RepoResult>)> {
    let backend = Backend::new(&ctx.db, (&ctx.config.pmr_git_root).into());
    let handle = backend.git_handle(workspace_id).await?;

    let pathinfo = match handle.pathinfo(None, None) {
        Ok(result) => (
            Some(format!("{}", result.commit.id())),
            Some(result.into()),
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
*/

async fn api_workspace_pathinfo(
    ctx: Extension<AppContext>,
    workspace_id: i64,
    commit_id: Option<String>,
    path: Option<String>,
) -> Result<Json<RepoResult>> {
    let backend = Backend::new(&ctx.db, (&ctx.config.pmr_git_root).into());
    let handle = backend.git_handle(workspace_id).await?;

    let result = match handle.pathinfo(
        commit_id.as_deref(),
        path.as_deref(),
    ) {
        Ok(pathinfo) => Ok(Json(pathinfo.into())),
        Err(e) => {
            // TODO log the URI triggering these messages?
            log::info!("handle.pathinfo error: {:?}", e);
            Err(Error::NotFound)
        }
    };
    result
}

pub async fn api_workspace_pathinfo_workspace_id(
    ctx: Extension<AppContext>,
    Path(workspace_id): Path<i64>,
) -> Result<Json<RepoResult>> {
    api_workspace_pathinfo(ctx, workspace_id, None, None).await
}

pub async fn api_workspace_pathinfo_workspace_id_commit_id(
    ctx: Extension<AppContext>,
    Path((workspace_id, commit_id)): Path<(i64, String)>,
) -> Result<Json<RepoResult>> {
    api_workspace_pathinfo(ctx, workspace_id, Some(commit_id), None).await
}


pub async fn api_workspace_pathinfo_workspace_id_commit_id_path(
    ctx: Extension<AppContext>,
    Path((workspace_id, commit_id, path)): Path<(i64, String, String)>,
) -> Result<Json<RepoResult>> {
    api_workspace_pathinfo(ctx, workspace_id, Some(commit_id), Some(path)).await
}

