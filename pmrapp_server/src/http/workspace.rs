use axum::{
    Json,
    Router,
    extract::{Extension, Path},
    http::header,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use pmrcore::{
    repo::{
        PathObjectInfo,
        RemoteInfo,
    },
};
use pmrrepo::{
    backend::Backend,
    handle::GitResultTarget,
};
use std::io::Write;

use crate::http::{
    api,
    AppContext,
    Error,
    Html,
    page,
};
use pmrapp_client::App;
use pmrapp_client::sauron::Render;


pub fn router() -> Router {
    Router::new()
        .route("/", get(render_workspace_listing))
        .route("/:workspace_id/", get(render_workspace))
        .route("/:workspace_id/file/:commit_id/", get(render_workspace_pathinfo_workspace_id_commit_id))
        .route("/:workspace_id/file/:commit_id/*path", get(render_workspace_pathinfo_workspace_id_commit_id_path))
        .route("/:workspace_id/raw/:commit_id/*path", get(raw_workspace_pathinfo_workspace_id_commit_id_path))
}

async fn render_workspace_listing(ctx: Extension<AppContext>) -> Response {
    match api::workspace::api_workspace(ctx).await {
        Ok(Json(workspace_listing)) => {
            let app = App::with_workspace_listing(workspace_listing);
            let content = page::index(&app).render_to_string();
            Html(content).into_response()
        },
        Err(e) => Error::from(e).into_response()
    }
}

async fn render_workspace(
    ctx: Extension<AppContext>,
    path: Path<i64>,
) -> Response {
    let workspace_id = path.0;
    match api::workspace::api_workspace_top(ctx, path).await {
        Ok(Json(repo_result)) => {
            let app = App::with_workspace_top(workspace_id, repo_result);
            let content = page::index(&app).render_to_string();
            Html(content).into_response()
        },
        Err(e) => Error::from(e).into_response()
    }
}

async fn render_workspace_pathinfo_workspace_id_commit_id(
    ctx: Extension<AppContext>,
    path: Path<(i64, String)>,
) -> Response {
    let workspace_id = path.0.0;
    let commit_id = path.1.clone();
    match api::workspace::api_workspace_pathinfo_workspace_id_commit_id(ctx, path).await {
        Ok(Json(path_info)) => {
            // XXX instead of None we have an empty string for path...
            let app = App::with_workspace_pathinfo(workspace_id, commit_id, "".to_string(), path_info);
            let content = page::index(&app).render_to_string();
            Html(content).into_response()
        },
        Err(e) => Error::from(e).into_response()
    }
}

async fn render_workspace_pathinfo_workspace_id_commit_id_path(
    ctx: Extension<AppContext>,
    path: Path<(i64, String, String)>,
) -> Response {
    let workspace_id = path.0.0;
    let commit_id = path.1.clone();
    let filepath = path.2.clone();
    match api::workspace::api_workspace_pathinfo_workspace_id_commit_id_path(ctx, path).await {
        Ok(Json(path_info)) => {
            let app = App::with_workspace_pathinfo(workspace_id, commit_id, filepath, path_info);
            let content = page::index(&app).render_to_string();
            Html(content).into_response()
        },
        Err(e) => Error::from(e).into_response()
    }
}

async fn raw_workspace_pathinfo_workspace_id_commit_id_path(
    ctx: Extension<AppContext>,
    path: Path<(i64, String, String)>,
) -> Response {
    let workspace_id = path.0.0;
    let commit_id = path.1.clone();
    let filepath = path.2.clone();

    let backend = Backend::new(&ctx.db, (&ctx.config.pmr_repo_root).into());
    let handle = match backend.git_handle(workspace_id).await {
        Ok(handle) => handle,
        Err(e) => return Error::from(e).into_response()
    };

    let result = match handle.pathinfo(
        Some(&commit_id),
        Some(&filepath),
    ) {
        Ok(result) => {
            let mut buffer = <Vec<u8>>::new();
            // The following is a !Send Future (async) so....
            // handle.stream_result_blob(&mut blob, &result).await?;
            // Ok(blob)

            match &result.target() {
                GitResultTarget::Object(object) => {
                    let info: PathObjectInfo = object.into();
                    match info {
                        PathObjectInfo::FileInfo(info) => {
                            // possible to avoid copying these bytes?
                            match (&mut buffer).write(&object.object.data) {
                                Ok(_) => Ok((
                                    [(header::CONTENT_TYPE, info.mime_type)],
                                    buffer
                                ).into_response()),
                                Err(_) => Err(Error::Error),
                            }
                        },
                        _ => {
                            log::info!("failed to get blob from object");
                            Err(Error::NotFound)
                        }
                    }
                }
                GitResultTarget::RemoteInfo(RemoteInfo { location, commit, subpath, .. }) => {
                    // XXX this should be a redirect
                    Ok(Redirect::temporary(
                        &format!("{}/raw/{}/{}", location, commit, subpath)
                    ).into_response())
                },
            }
        },
        Err(e) => {
            // TODO log the URI triggering these messages?
            log::info!("handle.pathinfo error: {:?}", e);
            Err(Error::NotFound)
        }
    };
    result.unwrap_or_else(|e| Error::from(e).into_response())
}
