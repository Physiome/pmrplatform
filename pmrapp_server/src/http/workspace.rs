use axum::{
    Json,
    Router,
    extract::{Extension, Path},
    http::header,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use pmrmodel_base::{
    git::PathObject,
    workspace::traits::WorkspaceBackend,
};
use pmrrepo::git::{
    Kind,
    GitResultTarget,
    PmrBackendWR,
};
use std::{
    io::Write,
    path::PathBuf,
};

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
    match api::workspace::api_workspace_top_ssr(ctx, path).await {
        Ok((record, path_info)) => {
            let app = App::with_workspace_top(workspace_id, record, path_info);
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

    let workspace = match WorkspaceBackend::get_workspace_by_id(&ctx.backend, workspace_id).await {
        Ok(workspace) => workspace,
        Err(_) => return Error::NotFound.into_response(),
    };
    let pmrbackend = match PmrBackendWR::new(
        &ctx.backend,
        PathBuf::from(&ctx.config.pmr_git_root),
        &workspace
    ) {
        Ok(pmrbackend) => pmrbackend,
        Err(e) => return Error::from(e).into_response()
    };

    let result = match pmrbackend.pathinfo(
        Some(&commit_id),
        Some(&filepath),
    ) {
        Ok(result) => {
            let mut buffer = <Vec<u8>>::new();
            // The following is a !Send Future (async) so....
            // pmrbackend.stream_result_blob(&mut blob, &result).await?;
            // Ok(blob)

            match &result.target {
                GitResultTarget::Object(object) => match object.kind {
                    Kind::Blob => {
                        let path_object = <Option<PathObject>>::from(&result);
                        match (path_object, object.kind) {
                            (Some(PathObject::FileInfo(info)), Kind::Blob) => {
                                // possible to avoid copying these bytes?
                                match (&mut buffer).write(&object.data) {
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
                    },
                    _ => {
                        log::info!("target is not a git blob");
                        Err(Error::NotFound)
                    },
                },
                GitResultTarget::SubRepoPath { location, commit, path } => {
                    // XXX this should be a redirect
                    Ok(Redirect::temporary(
                        &format!("{}/raw/{}/{}", location, commit, path)
                    ).into_response())
                },
            }
        },
        Err(e) => {
            // TODO log the URI triggering these messages?
            log::info!("pmrbackend.pathinfo error: {:?}", e);
            Err(Error::NotFound)
        }
    };
    result.unwrap_or_else(|e| Error::from(e).into_response())
}
