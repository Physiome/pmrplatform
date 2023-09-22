use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ExposureFileViewTaskTemplate {
    #[serde(default)]
    pub id: i64,
    pub exposure_file_id: i64,
    pub view_task_template_id: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ExposureFileViewTask {
    #[serde(default)]
    pub id: i64,
    pub exposure_file_view_id: i64,
    pub view_task_template_id: i64,
    #[serde(default)]
    pub task_id: Option<i64>,
    #[serde(default)]
    pub created_ts: Option<i64>,
    #[serde(default)]
    pub ready: bool,
}

pub mod traits;
