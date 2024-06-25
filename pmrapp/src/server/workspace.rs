use axum::{
    Extension,
    extract::Path,
    response::{
        IntoResponse,
        Redirect,
        Response,
    },
};
use http::header;
use pmrcore::{
    platform::{
        MCPlatform,
        TMPlatform,
    },
    repo::{
        PathObjectInfo,
        RemoteInfo,
    },
};
use pmrctrl::platform::Platform;
use pmrrepo::handle::GitResultTarget;
use std::io::Write;

use crate::error_template::AppError;


pub async fn raw_workspace_download<MCP, TMP>(
    platform: Extension<Platform<MCP, TMP>>,
    path: Path<(i64, String, String)>,
) -> Response
where
    MCP: MCPlatform + Clone + Sized + Send + Sync + 'static,
    TMP: TMPlatform + Clone + Sized + Send + Sync + 'static,
{

    let workspace_id = path.0.0;
    let commit_id = path.1.clone();
    let filepath = path.2.clone();

    let backend = platform.repo_backend();
    let handle = match backend.git_handle(workspace_id).await {
        Ok(handle) => handle,
        Err(_) => return AppError::InternalServerError.into_response()
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
                                Err(_) => Err(AppError::InternalServerError),
                            }
                        },
                        _ => {
                            // log::info!("failed to get blob from object");
                            Err(AppError::NotFound)
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
        Err(_) => {
            // TODO log the URI triggering these messages?
            // log::info!("handle.pathinfo error: {:?}", e);
            Err(AppError::NotFound)
        }
    };
    result.unwrap_or_else(|e| AppError::from(e).into_response())
}

