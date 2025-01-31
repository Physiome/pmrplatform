use axum::{
    Extension,
    extract::Path,
    response::{
        IntoResponse,
        Redirect,
        Response,
    },
};
use axum_login::AuthSession;
use http::header;
use pmrac::Platform as ACPlatform;
use pmrcore::{
    repo::{
        PathObjectInfo,
        RemoteInfo,
    },
};
use pmrctrl::platform::Platform;
use pmrrepo::handle::GitResultTarget;
use std::io::Write;

use crate::{
    error::AppError,
    server::ac::Session,
};

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
            // The following is a !Send Future (async) so....
            // handle.stream_result_blob(&mut blob, &result).await?;
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

