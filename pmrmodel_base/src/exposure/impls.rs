use std::ops::{Deref, DerefMut};
use crate::exposure::*;

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

impl<'a> From<Vec<ExposureFileRef<'a>>> for ExposureFileRefs<'a> {
    fn from(args: Vec<ExposureFileRef<'a>>) -> Self {
        Self(args)
    }
}

impl<'a, const N: usize> From<[ExposureFileRef<'a>; N]> for ExposureFileRefs<'a> {
    fn from(args: [ExposureFileRef<'a>; N]) -> Self {
        Self(args.into())
    }
}

impl<'a> Deref for ExposureFileRefs<'a> {
    type Target = Vec<ExposureFileRef<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
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

impl traits::Exposure for Exposure {
    fn id(&self) -> i64 {
        self.id
    }
    fn workspace_id(&self) -> i64 {
        self.workspace_id
    }
    fn workspace_tag_id(&self) -> Option<i64> {
        self.workspace_tag_id
    }
    fn commit_id(&self) -> &str {
        self.commit_id.as_ref()
    }
    fn created_ts(&self) -> i64 {
        self.created_ts
    }
    fn default_file_id(&self) -> Option<i64> {
        self.default_file_id
    }
}

impl traits::Exposure for ExposureRef<'_> {
    fn id(&self) -> i64 {
        self.inner.id
    }
    fn workspace_id(&self) -> i64 {
        self.inner.workspace_id
    }
    fn workspace_tag_id(&self) -> Option<i64> {
        self.inner.workspace_tag_id
    }
    fn commit_id(&self) -> &str {
        self.inner.commit_id.as_ref()
    }
    fn created_ts(&self) -> i64 {
        self.inner.created_ts
    }
    fn default_file_id(&self) -> Option<i64> {
        self.inner.default_file_id
    }
}

impl traits::ExposureFile for ExposureFile {
    fn id(&self) -> i64 {
        self.id
    }
    fn exposure_id(&self) -> i64 {
        self.exposure_id
    }
    fn workspace_file_path(&self) -> &str {
        self.workspace_file_path.as_ref()
    }
    fn default_view_id(&self) -> Option<i64> {
        self.default_view_id
    }
    // pub views: Option<ExposureFileViews>,
}

impl traits::ExposureFile for ExposureFileRef<'_> {
    fn id(&self) -> i64 {
        self.inner.id
    }
    fn exposure_id(&self) -> i64 {
        self.inner.exposure_id
    }
    fn workspace_file_path(&self) -> &str {
        self.inner.workspace_file_path.as_ref()
    }
    fn default_view_id(&self) -> Option<i64> {
        self.inner.default_view_id
    }
    // pub views: Option<ExposureFileViews>,
}

impl traits::ExposureFileView for ExposureFileView {
    fn id(&self) -> i64 {
        self.id
    }
    fn exposure_file_id(&self) -> i64 {
        self.exposure_file_id
    }
    fn view_key(&self) -> &str {
        self.view_key.as_ref()
    }
}

impl traits::ExposureFileView for ExposureFileViewRef<'_> {
    fn id(&self) -> i64 {
        self.inner.id
    }
    fn exposure_file_id(&self) -> i64 {
        self.inner.exposure_file_id
    }
    fn view_key(&self) -> &str {
        self.inner.view_key.as_ref()
    }
}
