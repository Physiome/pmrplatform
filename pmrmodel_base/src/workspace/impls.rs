use async_trait::async_trait;
use std::ops::{
    Deref,
    DerefMut,
};
use crate::error::ValueError;
use crate::exposure::{
    traits::Backend,
    ExposureRefs,
};
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

impl<'a, B: Backend + Sized> From<Vec<WorkspaceRef<'a, B>>> for WorkspaceRefs<'a, B> {
    fn from(args: Vec<WorkspaceRef<'a, B>>) -> Self {
        Self(args)
    }
}

impl<'a, B: Backend + Sized, const N: usize> From<[WorkspaceRef<'a, B>; N]> for WorkspaceRefs<'a, B> {
    fn from(args: [WorkspaceRef<'a, B>; N]) -> Self {
        Self(args.into())
    }
}

impl<'a, B: Backend + Sized> Deref for WorkspaceRefs<'a, B> {
    type Target = Vec<WorkspaceRef<'a, B>>;

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
    async fn exposures(&'a self) -> Result<&Exposures, ValueError> {
        Ok(self.exposures.as_ref().ok_or(ValueError::Uninitialized)?)
    }
}

#[async_trait]
impl<'a, B: Backend + Sized + Sync> traits::Workspace<'a, ExposureRefs<'a, B>> for WorkspaceRef<'a, B> {
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
    async fn exposures(&'a self) -> Result<&'a ExposureRefs<'a, B>, ValueError> {
        match self.exposures.get() {
            Some(exposures) => Ok(exposures),
            None => {
                self.exposures.set(
                    self.backend.get_exposures(self.inner.id).await?
                ).unwrap_or_else(|_| log::warn!(
                    "concurrent call to the same WorkspaceRef.exposures() \
                    instance accessed backend"
                ));
                Ok(self.exposures.get()
                    .expect("exposures should have been set just now!"))
            }
        }
    }
}
