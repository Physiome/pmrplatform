use async_trait::async_trait;
use std::ops::{
    Deref,
    DerefMut,
};
use crate::error::{
    Error,
    ValueError,
};
use crate::exposure::ExposureRefs;
use crate::workspace::*;

impl From<Vec<Workspace>> for Workspaces {
    fn from(args: Vec<Workspace>) -> Self {
        Self(args)
    }
}

impl<const N: usize> From<[Workspace; N]> for Workspaces {
    fn from(args: [Workspace; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for Workspaces {
    type Target = Vec<Workspace>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Workspaces {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Workspaces {
    type Item = Workspace;
    type IntoIter = std::vec::IntoIter<Workspace>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> From<Vec<WorkspaceRef<'a>>> for WorkspaceRefs<'a> {
    fn from(args: Vec<WorkspaceRef<'a>>) -> Self {
        Self(args)
    }
}

impl<'a, const N: usize> From<[WorkspaceRef<'a>; N]> for WorkspaceRefs<'a> {
    fn from(args: [WorkspaceRef<'a>; N]) -> Self {
        Self(args.into())
    }
}

impl<'a> Deref for WorkspaceRefs<'a> {
    type Target = Vec<WorkspaceRef<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<'a> traits::Workspace<'a, Exposures> for Workspace {
    fn id(&self) -> i64 {
        self.id
    }
    fn url(&self) -> &str {
        self.url.as_ref()
    }
    fn superceded_by_id(&self) -> Option<i64> {
        self.superceded_by_id
    }
    fn description(&self) -> Option<&str> {
        self.description.as_ref().map(|v| v.as_ref())
    }
    fn long_description(&self) -> Option<&str> {
        self.long_description.as_ref().map(|v| v.as_ref())
    }
    fn created_ts(&self) -> i64 {
        self.created_ts
    }
    async fn exposures(&'a self) -> Result<&Exposures, Error> {
        Ok(self.exposures.as_ref().ok_or(ValueError::Uninitialized)?)
    }
}

#[async_trait]
impl<'a> traits::Workspace<'a, ExposureRefs<'a>> for WorkspaceRef<'a> {
    fn id(&self) -> i64 {
        self.inner.id
    }
    fn url(&self) -> &str {
        self.inner.url.as_ref()
    }
    fn superceded_by_id(&self) -> Option<i64> {
        self.inner.superceded_by_id
    }
    fn description(&self) -> Option<&str> {
        self.inner.description.as_ref().map(|v| v.as_ref())
    }
    fn long_description(&self) -> Option<&str> {
        self.inner.long_description.as_ref().map(|v| v.as_ref())
    }
    fn created_ts(&self) -> i64 {
        self.inner.created_ts
    }
    async fn exposures(&'a self) -> Result<&'a ExposureRefs<'a>, Error> {
        match self.exposures.get() {
            Some(exposures) => Ok(exposures),
            None => {
                self.exposures.set(
                    self.platform.get_exposures(self.inner.id).await?
                ).unwrap_or_else(|_| log::warn!(
                    "concurrent call to the same WorkspaceRef.exposures() \
                    instance accessed platform"
                ));
                Ok(self.exposures.get()
                    .expect("exposures should have been set just now!"))
            }
        }
    }
}
