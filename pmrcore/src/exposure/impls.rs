use async_trait::async_trait;
use std::ops::{Deref, DerefMut};
use crate::error::{
    Error,
    ValueError,
};
use crate::exposure::*;
use crate::platform::MCPlatform;
use crate::workspace::{
    Workspace,
    WorkspaceRef,
};

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

impl IntoIterator for Exposures {
    type Item = Exposure;
    type IntoIter = std::vec::IntoIter<Exposure>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, P: MCPlatform + Sized> From<Vec<ExposureRef<'a, P>>> for ExposureRefs<'a, P> {
    fn from(args: Vec<ExposureRef<'a, P>>) -> Self {
        Self(args)
    }
}

impl<'a, P: MCPlatform + Sized, const N: usize> From<[ExposureRef<'a, P>; N]> for ExposureRefs<'a, P> {
    fn from(args: [ExposureRef<'a, P>; N]) -> Self {
        Self(args.into())
    }
}

impl<'a, P: MCPlatform + Sized> Deref for ExposureRefs<'a, P> {
    type Target = Vec<ExposureRef<'a, P>>;

    fn deref(&self) -> &Self::Target {
        &self.0
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

impl<'a, P: MCPlatform + Sized> From<Vec<ExposureFileRef<'a, P>>> for ExposureFileRefs<'a, P> {
    fn from(args: Vec<ExposureFileRef<'a, P>>) -> Self {
        Self(args)
    }
}

impl<'a, P: MCPlatform + Sized, const N: usize> From<[ExposureFileRef<'a, P>; N]> for ExposureFileRefs<'a, P> {
    fn from(args: [ExposureFileRef<'a, P>; N]) -> Self {
        Self(args.into())
    }
}

impl<'a, P: MCPlatform + Sized> Deref for ExposureFileRefs<'a, P> {
    type Target = Vec<ExposureFileRef<'a, P>>;

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

impl<'a, P: MCPlatform + Sized> From<Vec<ExposureFileViewRef<'a, P>>> for ExposureFileViewRefs<'a, P> {
    fn from(args: Vec<ExposureFileViewRef<'a, P>>) -> Self {
        Self(args)
    }
}

impl<'a, P: MCPlatform + Sized, const N: usize> From<[ExposureFileViewRef<'a, P>; N]> for ExposureFileViewRefs<'a, P> {
    fn from(args: [ExposureFileViewRef<'a, P>; N]) -> Self {
        Self(args.into())
    }
}

impl<'a, P: MCPlatform + Sized> Deref for ExposureFileViewRefs<'a, P> {
    type Target = Vec<ExposureFileViewRef<'a, P>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<'a> traits::Exposure<'a, ExposureFiles, Workspace> for Exposure {
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
    async fn files(&'a self) -> Result<&ExposureFiles, Error> {
        Ok(self.files.as_ref().ok_or(ValueError::Uninitialized)?)
    }
    async fn workspace(&'a self) -> Result<&Workspace, Error> {
        // reference to parent is not provided, so simply uninitialized
        Err(Error::Value(ValueError::Uninitialized))
    }
}

#[async_trait]
impl<'a, P: MCPlatform + Sized + Sync>
    traits::Exposure<'a, ExposureFileRefs<'a, P>, WorkspaceRef<'a, P>>
for ExposureRef<'a, P> {
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
    async fn files(&'a self) -> Result<&'a ExposureFileRefs<'a, P>, Error> {
        match self.files.get() {
            Some(files) => Ok(files),
            None => {
                self.files.set(
                    self.platform.get_exposure_files(self.inner.id).await?
                ).unwrap_or_else(|_| log::warn!(
                    "concurrent call to the same ExposureRef.files() \
                    instance accessed platform"
                ));
                Ok(self.files.get().expect("files just been set!"))
            }
        }
    }
    async fn workspace(
        &'a self
    ) -> Result<&'a WorkspaceRef<'a, P>, Error> {
        match self.parent.get() {
            Some(parent) => Ok(parent),
            None => {
                self.parent.set(
                    self.platform.get_workspace(self.inner.workspace_id).await?
                ).unwrap_or_else(|_| log::warn!(
                    "concurrent call to the same ExposureRef.workspace() \
                    instance accessed platform"
                ));
                Ok(self.parent.get().expect("parent just been set!"))
            }
        }
    }
}

#[async_trait]
impl<'a> traits::ExposureFile<'a, ExposureFileViews, Exposure> for ExposureFile {
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
    async fn views(&'a self) -> Result<&ExposureFileViews, Error> {
        Ok(self.views.as_ref().ok_or(ValueError::Uninitialized)?)
    }
    async fn exposure(&'a self) -> Result<&'a Exposure, Error> {
        // reference to parent is not provided, so simply uninitialized
        Err(Error::Value(ValueError::Uninitialized))
    }
}

#[async_trait]
impl<'a, P: MCPlatform + Sized + Sync>
    traits::ExposureFile<'a, ExposureFileViewRefs<'a, P>, ExposureRef<'a, P>>
for ExposureFileRef<'a, P> {
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
    async fn views(
        &'a self
    ) -> Result<&'a ExposureFileViewRefs<'a, P>, Error> {
        match self.views.get() {
            Some(views) => Ok(views),
            None => {
                self.views.set(
                    self.platform.get_exposure_file_views(self.inner.id).await?
                ).unwrap_or_else(|_| log::warn!(
                    "concurrent call to the same ExposureFileRef.views() \
                    instance accessed platform"
                ));
                Ok(self.views.get().expect("views just been set!"))
            }
        }
    }
    async fn exposure(
        &'a self
    ) -> Result<&'a ExposureRef<'a, P>, Error> {
        match self.parent.get() {
            Some(parent) => Ok(parent),
            None => {
                self.parent.set(
                    self.platform.get_exposure(self.inner.exposure_id).await?
                ).unwrap_or_else(|_| log::warn!(
                    "concurrent call to the same ExposureFileRef.parent() \
                    instance accessed platform"
                ));
                Ok(self.parent.get().expect("parent just been set!"))
            }
        }
    }
}

#[async_trait]
impl<'a> traits::ExposureFileView<'a, ExposureFile> for ExposureFileView {
    fn id(&self) -> i64 {
        self.id
    }
    fn exposure_file_id(&self) -> i64 {
        self.exposure_file_id
    }
    fn view_task_template_id(&self) -> i64 {
        self.view_task_template_id
    }
    fn exposure_file_view_task_id(&self) -> Option<i64> {
        self.exposure_file_view_task_id
    }
    fn view_key(&self) -> Option<&str> {
        self.view_key.as_ref().map(|x| x.as_ref())
    }
    fn updated_ts(&self) -> i64 {
        self.updated_ts
    }
    async fn exposure_file(&'a self) -> Result<&'a ExposureFile, Error> {
        // reference to parent is not provided, so simply uninitialized
        Err(Error::Value(ValueError::Uninitialized))
    }
}

#[async_trait]
impl<'a, P: MCPlatform + Sized + Sync>
    traits::ExposureFileView<'a, ExposureFileRef<'a, P>>
for ExposureFileViewRef<'a, P> {
    fn id(&self) -> i64 {
        self.inner.id
    }
    fn exposure_file_id(&self) -> i64 {
        self.inner.exposure_file_id
    }
    fn view_task_template_id(&self) -> i64 {
        self.inner.view_task_template_id
    }
    fn exposure_file_view_task_id(&self) -> Option<i64> {
        self.inner.exposure_file_view_task_id
    }
    fn view_key(&self) -> Option<&str> {
        self.inner.view_key.as_ref().map(|x| x.as_ref())
    }
    fn updated_ts(&self) -> i64 {
        self.inner.updated_ts
    }
    async fn exposure_file(
        &'a self
    ) -> Result<&'a ExposureFileRef<'a, P>, Error> {
        match self.parent.get() {
            Some(parent) => Ok(parent),
            None => {
                self.parent.set(
                    self.platform.get_exposure_file(self.inner.exposure_file_id)
                        .await?
                ).unwrap_or_else(|_| log::warn!(
                    "concurrent call to the same ExposureFileViewRef.parent() \
                    instance accessed platform"
                ));
                Ok(self.parent.get().expect("parent just been set!"))
            }
        }
    }
}
