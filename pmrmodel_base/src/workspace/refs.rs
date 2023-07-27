use std::sync::OnceLock;
use crate::{
    exposure,
    platform::Platform,
    workspace::{
        Workspace,
        Workspaces,
    },
};

pub struct WorkspaceRef<'a, P: Platform + Sized> {
    pub(super) inner: Workspace,
    pub(super) exposures: OnceLock<exposure::ExposureRefs<'a, P>>,
    pub(super) platform: &'a P,
}

pub struct WorkspaceRefs<'a, P: Platform + Sized>(pub(super) Vec<WorkspaceRef<'a, P>>);

impl Workspace {
    pub(crate) fn bind<'a, P: Platform + Sized>(
        self,
        platform: &'a P,
    ) -> WorkspaceRef<'a, P> {
        WorkspaceRef {
            inner: self,
            exposures: OnceLock::new(),
            platform: platform,
        }
    }
}

impl Workspaces {
    pub(crate) fn bind<'a, P: Platform + Sized>(
        self,
        platform: &'a P,
    ) -> WorkspaceRefs<'a, P> {
        self.0
            .into_iter()
            .map(|v| v.bind(platform))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<P: Platform + Sized> WorkspaceRef<'_, P> {
    pub fn into_inner(self) -> Workspace {
        self.inner
    }
}
