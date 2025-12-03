use axum::{
    Extension,
    Json,
    extract::Path,
    response::{
        IntoResponse,
        Redirect,
        Response,
    },
};
use axum_extra::extract::Host;
use axum_login::AuthSession;
use collection_json::{
    builder::LinkBuilder,
    Collection,
};
use http::header;
use pmrac::Platform as ACPlatform;
use pmrcore::repo::{
    PathObjectInfo,
    RemoteInfo,
};
use pmrctrl::platform::Platform;
use pmrrepo::handle::GitResultTarget;
use std::io::Write;

use crate::{
    app::id::Id,
    error::AppError,
    server::{
        self,
        ac::Session,
    },
};

pub async fn resolve_id(id: Id) -> Result<i64, AppError> {
    server::resolve_id("workspace", id).await
}

pub async fn collection_json_workspace(
    platform: Extension<Platform>,
    session: Extension<AuthSession<ACPlatform>>,
    Host(hostname): Host,
) -> Result<Json<Collection>, AppError> {
    // TODO figure out how or whether to derive this
    let proto = "http";
    Session::from(session)
        .enforcer("/workspace/", "").await?;
    let collection = Collection::new(&format!("{proto}://{hostname}/workspace/"))
        .map_err(|_| AppError::InternalServerError)?;
    let links = platform.mc_platform
        .list_aliased_workspaces()
        .await
        .map_err(|_| AppError::InternalServerError)?
        .into_iter()
        .filter_map(|entry| {
            let workspace = entry.entity.into_inner();
            // TODO log entries that don't `.build_with()` correctly?
            LinkBuilder::new(entry.alias, "bookmark".to_string())
                .prompt(workspace.description
                    .unwrap_or(format!("Workspace {}", workspace.id))
                )
                .build_with(&collection)
                .ok()
        })
        .collect();
    Ok(Json(collection.links(links)))
}

pub async fn raw_aliased_workspace_download(
    platform: Extension<Platform>,
    session: Extension<AuthSession<ACPlatform>>,
    Path((workspace_alias, commit_id, filepath)): Path<(String, String, String)>,
) -> Result<Response, AppError> {
    let workspace_id = platform
        .mc_platform
        .resolve_alias("workspace", &workspace_alias)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or(AppError::NotFound)?;
    raw_workspace_download(
        platform,
        session,
        Path((workspace_id, commit_id, filepath)),
    ).await
}

pub async fn raw_workspace_download(
    platform: Extension<Platform>,
    session: Extension<AuthSession<ACPlatform>>,
    Path((workspace_id, commit_id, filepath)): Path<(i64, String, String)>,
) -> Result<Response, AppError> {
    Session::from(session)
        .enforcer(format!("/workspace/{workspace_id}/"), "").await?;
    let backend = platform.repo_backend();
    let handle = backend.git_handle(workspace_id).await
        .map_err(|_| AppError::InternalServerError)?;

    let result = match handle.pathinfo(
        Some(&commit_id),
        Some(&filepath),
    ) {
        Ok(result) => {
            let mut buffer = <Vec<u8>>::new();
            // This doesn't quite handle remote mimetype, perhaps upstream need to
            // provide a better method.
            // handle.stream_blob(&mut blob, &result).await?;
            // Ok(blob)

            match &result.target() {
                Some(GitResultTarget::Object(object)) => {
                    let info: PathObjectInfo = object.into();
                    match info {
                        PathObjectInfo::FileInfo(info) => {
                            // possible to avoid copying these bytes?
                            match (&mut buffer).write(&object.object.data) {
                                Ok(_) => Ok((
                                    // TODO include last modified info for the file
                                    // at least for the commit, but ideally when the
                                    // file actually changed.
                                    [(header::CONTENT_TYPE, info.mime_type)],
                                    buffer
                                ).into_response()),
                                Err(_) => Err(AppError::InternalServerError),
                            }
                        },
                        _ => {
                            // log::info!("failed to get blob from object");
                            Err(AppError::NotFound)
                        }
                    }
                }
                Some(GitResultTarget::RemoteInfo(RemoteInfo { location, commit, subpath, .. })) => {
                    Ok(Redirect::temporary(
                        &format!("{}/raw/{}/{}", location, commit, subpath)
                    ).into_response())
                },
                None => Err(AppError::NotFound),
            }
        },
        Err(_) => {
            // TODO log the URI triggering these messages?
            // log::info!("handle.pathinfo error: {:?}", e);
            Err(AppError::NotFound)
        }
    };
    Ok(result.unwrap_or_else(|e| AppError::from(e).into_response()))
}

