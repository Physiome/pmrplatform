use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};
#[cfg(feature = "display")]
use std::fmt::{
    Display,
    Formatter,
    Result,
};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Exposure {
    pub id: i64,
    pub workspace_id: i64,
    pub workspace_tag_id: Option<i64>,
    pub commit_id: String,
    pub created_ts: i64,
    pub root_exposure_file_id: Option<i64>,

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
    pub default_view: Option<String>,

    // derived fields
    pub views: Option<ExposureFileViews>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ExposureFiles(Vec<ExposureFile>);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ExposureFileView {
    pub id: i64,
    pub exposure_file_id: i64,
    pub view_key: String,
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
impl Display for Exposure {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{} - {} - {} - {}",
            self.id,
            self.workspace_id,
            self.workspace_tag_id,
            &self.commit_id,
        )
    }
}

impl From<Vec<Exposure>> for Exposures {
    fn from(args: Vec<Exposure>) -> Self {
        Self(args)
    }
}

impl<const N: usize> From<[Exposure; N]> for Exposures {
    fn from(args: [Exposure; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for Exposures {
    type Target = Vec<Exposure>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Exposures {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<ExposureFile>> for ExposureFiles {
    fn from(args: Vec<ExposureFile>) -> Self {
        Self(args)
    }
}

impl<const N: usize> From<[ExposureFile; N]> for ExposureFiles {
    fn from(args: [ExposureFile; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for ExposureFiles {
    type Target = Vec<ExposureFile>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ExposureFiles {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<ExposureFileView>> for ExposureFileViews {
    fn from(args: Vec<ExposureFileView>) -> Self {
        Self(args)
    }
}

impl<const N: usize> From<[ExposureFileView; N]> for ExposureFileViews {
    fn from(args: [ExposureFileView; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for ExposureFileViews {
    type Target = Vec<ExposureFileView>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ExposureFileViews {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
