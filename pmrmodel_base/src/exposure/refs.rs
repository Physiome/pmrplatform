use crate::exposure::{
    Exposure,
    ExposureFile,
    ExposureFileView,
    traits,
};

pub struct ExposureRef<'a> {
    pub(super) inner: Exposure,
    // pub(super) inner_files: ExposureFileRefs<'a>,
    pub(super) backend: &'a dyn traits::Backend,
}

pub struct ExposureRefs<'a>(Vec<ExposureRef<'a>>);

pub struct ExposureFileRef<'a> {
    pub(super) inner: ExposureFile,
    // pub(super) inner_views: ExposureFileViewRefs<'a>,
    pub(super) backend: &'a dyn traits::Backend,
}

pub struct ExposureFileRefs<'a>(pub(super) Vec<ExposureFileRef<'a>>);

pub struct ExposureFileViewRef<'a> {
    pub(super) inner: ExposureFileView,
    pub(super) backend: &'a dyn traits::Backend,
}

pub struct ExposureFileViewRefs<'a>(pub(super) Vec<ExposureFileViewRef<'a>>);

impl Exposure {
    pub(super) fn bind<'a>(
        self,
        backend: &'a dyn traits::Backend,
    ) -> ExposureRef<'a> {
        ExposureRef {
            inner: self,
            backend: backend,
        }
    }
}

impl ExposureRef<'_> {
    pub fn into_inner(self) -> Exposure {
        self.inner
    }
}

impl ExposureFile {
    pub(super) fn bind<'a>(
        self,
        backend: &'a dyn traits::Backend,
    ) -> ExposureFileRef<'a> {
        ExposureFileRef {
            inner: self,
            backend: backend,
        }
    }
}

impl ExposureFileRef<'_> {
    pub fn into_inner(self) -> ExposureFile {
        self.inner
    }
}

impl ExposureFileView {
    pub(super) fn bind<'a>(
        self,
        backend: &'a dyn traits::Backend,
    ) -> ExposureFileViewRef<'a> {
        ExposureFileViewRef {
            inner: self,
            backend: backend,
        }
    }
}

impl ExposureFileViewRef<'_> {
    pub fn into_inner(self) -> ExposureFileView {
        self.inner
    }
}
