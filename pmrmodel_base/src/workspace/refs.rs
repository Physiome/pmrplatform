use std::sync::OnceLock;
use crate::{
    exposure,
    workspace::{
        Workspace,
        Workspaces,
    },
};

pub struct WorkspaceRef<'a, Backend: exposure::traits::Backend + Sized> {
    pub(super) inner: Workspace,
    pub(super) exposures: OnceLock<exposure::ExposureRefs<'a, Backend>>,
    // TODO if/when a platform that encapsulates all common thing,
    // change this reference to that.
    pub(super) backend: &'a Backend,
}

pub struct WorkspaceRefs<'a, B: exposure::traits::Backend + Sized>(pub(super) Vec<WorkspaceRef<'a, B>>);

impl Workspace {
    pub(crate) fn bind<'a, B: exposure::traits::Backend + Sized>(
        self,
        backend: &'a B,
    ) -> WorkspaceRef<'a, B> {
        WorkspaceRef {
            inner: self,
            exposures: OnceLock::new(),
            backend: backend,
        }
    }
}

impl Workspaces {
    pub(crate) fn bind<'a, B: exposure::traits::Backend + Sized>(
        self,
        backend: &'a B,
    ) -> WorkspaceRefs<'a, B> {
        self.0
            .into_iter()
            .map(|v| v.bind(backend))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<B: exposure::traits::Backend + Sized> WorkspaceRef<'_, B> {
    pub fn into_inner(self) -> Workspace {
        self.inner
    }
}
