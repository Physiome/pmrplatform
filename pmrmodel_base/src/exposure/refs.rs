use std::sync::OnceLock;
use crate::exposure::{
    Exposure,
    Exposures,
    ExposureFile,
    ExposureFiles,
    ExposureFileView,
    ExposureFileViews,
    traits,
};

pub struct ExposureRef<'a, Backend: traits::Backend + Sized> {
    pub(super) inner: Exposure,
    pub(super) files: OnceLock<ExposureFileRefs<'a, Backend>>,
    pub(super) backend: &'a Backend,
}

pub struct ExposureRefs<'a, B: traits::Backend + Sized>(pub(super) Vec<ExposureRef<'a, B>>);

pub struct ExposureFileRef<'a, Backend: traits::Backend + Sized> {
    pub(super) inner: ExposureFile,
    pub(super) views: OnceLock<ExposureFileViewRefs<'a, Backend>>,
    pub(super) backend: &'a Backend,
}

pub struct ExposureFileRefs<'a, B: traits::Backend + Sized>(pub(super) Vec<ExposureFileRef<'a, B>>);

pub struct ExposureFileViewRef<'a, Backend: traits::Backend + Sized> {
    pub(super) inner: ExposureFileView,
    pub(super) backend: &'a Backend,
}

pub struct ExposureFileViewRefs<'a, B: traits::Backend + Sized>(pub(super) Vec<ExposureFileViewRef<'a, B>>);

impl Exposure {
    pub(super) fn bind<'a, B: traits::Backend + Sized>(
        self,
        backend: &'a B,
    ) -> ExposureRef<'a, B> {
        ExposureRef {
            inner: self,
            // TODO verify that inner.files is also None?
            files: OnceLock::new(),
            backend: backend,
        }
    }
}

impl Exposures {
    pub(super) fn bind<'a, B: traits::Backend + Sized>(
        self,
        backend: &'a B,
    ) -> ExposureRefs<'a, B> {
        self.0
            .into_iter()
            .map(|v| v.bind(backend))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<B: traits::Backend + Sized> ExposureRef<'_, B> {
    pub fn into_inner(self) -> Exposure {
        self.inner
    }
}

impl ExposureFile {
    pub(super) fn bind<'a, B: traits::Backend + Sized>(
        self,
        backend: &'a B,
    ) -> ExposureFileRef<'a, B> {
        ExposureFileRef {
            inner: self,
            // TODO verify that inner.views is also None?
            views: OnceLock::new(),
            backend: backend,
        }
    }
}

impl ExposureFiles {
    pub(super) fn bind<'a, B: traits::Backend + Sized>(
        self,
        backend: &'a B,
    ) -> ExposureFileRefs<'a, B> {
        self.0
            .into_iter()
            .map(|v| v.bind(backend))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<B: traits::Backend + Sized> ExposureFileRef<'_, B> {
    pub fn into_inner(self) -> ExposureFile {
        self.inner
    }
}

impl ExposureFileView {
    pub(super) fn bind<'a, B: traits::Backend + Sized>(
        self,
        backend: &'a B,
    ) -> ExposureFileViewRef<'a, B> {
        ExposureFileViewRef {
            inner: self,
            backend: backend,
        }
    }
}

impl ExposureFileViews {
    pub(super) fn bind<'a, B: traits::Backend + Sized>(
        self,
        backend: &'a B,
    ) -> ExposureFileViewRefs<'a, B> {
        self.0
            .into_iter()
            .map(|v| v.bind(backend))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<B: traits::Backend + Sized> ExposureFileViewRef<'_, B> {
    pub fn into_inner(self) -> ExposureFileView {
        self.inner
    }
}
