use pmrmodel_base::workspace::{
    Workspaces,
    // Workspace,
};
use pmrmodel_base::repo::RepoResult;
use crate::error::ServerError;

pub async fn request_get_json<T: serde::de::DeserializeOwned>(
    url: &str,
) -> Result<T, ServerError> {
    log::trace!("request_get_json: {}", url);
    let response = reqwest::get(url).await?;
    Ok(response.json::<T>().await?)
}

pub async fn get_workspace_listing() -> Result<Workspaces, ServerError> {
    let url = format!("{}/api/workspace/", sauron::window().location().origin().expect("must have location"));
    Ok(request_get_json::<Workspaces>(&url).await?)
}

pub async fn get_workspace_top(workspace_id: &i64) -> Result<RepoResult, ServerError> {
    let url = format!("{}/api/workspace/{}/", sauron::window().location().origin().expect("must have location"), workspace_id);
    Ok(request_get_json::<RepoResult>(&url).await?)
}

pub async fn get_workspace_pathinfo(workspace_id: &i64, commit_id: &str, path: Option<&str>) -> Result<RepoResult, ServerError> {
    let url = format!("{}/api/workspace/{}/file/{}/{}",
        sauron::window().location().origin().expect("must have location"),
        workspace_id,
        commit_id,
        path.unwrap_or(""),
    );
    Ok(request_get_json::<RepoResult>(&url).await?)
}
