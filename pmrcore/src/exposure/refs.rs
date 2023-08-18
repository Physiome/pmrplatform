use std::sync::OnceLock;
use crate::exposure::{
    Exposure,
    Exposures,
    ExposureFile,
    ExposureFiles,
    ExposureFileView,
    ExposureFileViews,
};
use crate::platform::MCPlatform;
use crate::workspace::WorkspaceRef;

pub struct ExposureRef<'a, P: MCPlatform + Sized> {
    pub(super) inner: Exposure,
    pub(super) files: OnceLock<ExposureFileRefs<'a, P>>,
    pub(super) platform: &'a P,
    pub(super) parent: OnceLock<WorkspaceRef<'a, P>>,
}

pub struct ExposureRefs<'a, P: MCPlatform + Sized>(pub(super) Vec<ExposureRef<'a, P>>);

pub struct ExposureFileRef<'a, P: MCPlatform + Sized> {
    pub(super) inner: ExposureFile,
    pub(super) views: OnceLock<ExposureFileViewRefs<'a, P>>,
    pub(super) platform: &'a P,
    pub(super) parent: OnceLock<ExposureRef<'a, P>>,
}

pub struct ExposureFileRefs<'a, P: MCPlatform + Sized>(pub(super) Vec<ExposureFileRef<'a, P>>);

pub struct ExposureFileViewRef<'a, P: MCPlatform + Sized> {
    pub(super) inner: ExposureFileView,
    pub(super) platform: &'a P,
    pub(super) parent: OnceLock<ExposureFileRef<'a, P>>,
}

pub struct ExposureFileViewRefs<'a, P: MCPlatform + Sized>(pub(super) Vec<ExposureFileViewRef<'a, P>>);

impl Exposure {
    pub(crate) fn bind<'a, P: MCPlatform + Sized>(
        self,
        platform: &'a P,
    ) -> ExposureRef<'a, P> {
        ExposureRef {
            inner: self,
            // TODO verify that inner.files is also None?
            files: OnceLock::new(),
            platform: platform,
            parent: OnceLock::new(),
        }
    }
}

impl Exposures {
    pub(crate) fn bind<'a, P: MCPlatform + Sized>(
        self,
        platform: &'a P,
    ) -> ExposureRefs<'a, P> {
        self.0
            .into_iter()
            .map(|v| v.bind(platform))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<P: MCPlatform + Sized> ExposureRef<'_, P> {
    pub fn into_inner(self) -> Exposure {
        self.inner
    }
}

impl ExposureFile {
    pub(crate) fn bind<'a, P: MCPlatform + Sized>(
        self,
        platform: &'a P,
    ) -> ExposureFileRef<'a, P> {
        ExposureFileRef {
            inner: self,
            // TODO verify that inner.views is also None?
            views: OnceLock::new(),
            platform: platform,
            parent: OnceLock::new(),
        }
    }
}

impl ExposureFiles {
    pub(crate) fn bind<'a, P: MCPlatform + Sized>(
        self,
        platform: &'a P,
    ) -> ExposureFileRefs<'a, P> {
        self.0
            .into_iter()
            .map(|v| v.bind(platform))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<P: MCPlatform + Sized> ExposureFileRef<'_, P> {
    pub fn into_inner(self) -> ExposureFile {
        self.inner
    }
}

impl ExposureFileView {
    pub(crate) fn bind<'a, P: MCPlatform + Sized>(
        self,
        platform: &'a P,
    ) -> ExposureFileViewRef<'a, P> {
        ExposureFileViewRef {
            inner: self,
            platform: platform,
            parent: OnceLock::new(),
        }
    }
}

impl ExposureFileViews {
    pub(crate) fn bind<'a, P: MCPlatform + Sized>(
        self,
        platform: &'a P,
    ) -> ExposureFileViewRefs<'a, P> {
        self.0
            .into_iter()
            .map(|v| v.bind(platform))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<P: MCPlatform + Sized> ExposureFileViewRef<'_, P> {
    pub fn into_inner(self) -> ExposureFileView {
        self.inner
    }
}
