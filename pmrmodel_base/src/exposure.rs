use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Exposure {
    pub id: i64,
    pub workspace_id: i64,
    pub workspace_tag_id: Option<i64>,
    pub commit_id: String,
    pub created_ts: i64,
    pub default_file_id: Option<i64>,

    // derived fields
    pub files: Option<ExposureFiles>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Exposures(Vec<Exposure>);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ExposureFile {
    pub id: i64,
    pub exposure_id: i64,
    pub workspace_file_path: String,
    pub default_view_id: Option<i64>,

    // derived fields
    pub views: Option<ExposureFileViews>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ExposureFiles(Vec<ExposureFile>);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ExposureFileView {
    pub id: i64,
    pub exposure_file_id: i64,
    pub view_task_template_id: i64,
    pub view_key: Option<String>,
    pub updated_ts: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ExposureFileViews(Vec<ExposureFileView>);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ExposureFileViewTask {
    pub id: i64,
    pub exposure_file_view_id: i64,
    pub task_id: i64,
}

#[cfg(feature = "display")]
mod display;
mod impls;
mod refs;
pub mod traits;

pub use refs::{
    ExposureRef,
    ExposureRefs,
    ExposureFileRef,
    ExposureFileRefs,
    ExposureFileViewRef,
    ExposureFileViewRefs,
};
