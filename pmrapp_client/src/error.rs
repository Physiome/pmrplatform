#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}
