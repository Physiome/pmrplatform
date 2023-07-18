use crate::exposure::{
    Exposure,
    Exposures,
    ExposureFile,
    ExposureFiles,
    ExposureFileView,
    ExposureFileViews,
    traits,
};

pub struct ExposureRef<'a> {
    pub(super) inner: Exposure,
    pub(super) files: Option<ExposureFileRefs<'a>>,
    pub(super) backend: &'a dyn traits::Backend,
}

pub struct ExposureRefs<'a>(pub(super) Vec<ExposureRef<'a>>);

pub struct ExposureFileRef<'a> {
    pub(super) inner: ExposureFile,
    pub(super) views: Option<ExposureFileViewRefs<'a>>,
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
            // TODO verify that inner.files is also None?
            files: None,
            backend: backend,
        }
    }
}

impl Exposures {
    pub(super) fn bind<'a>(
        self,
        backend: &'a dyn traits::Backend,
    ) -> ExposureRefs<'a> {
        self.0
            .into_iter()
            .map(|v| v.bind(backend))
            .collect::<Vec<_>>()
            .into()
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
            // TODO verify that inner.views is also None?
            views: None,
            backend: backend,
        }
    }
}

impl ExposureFiles {
    pub(super) fn bind<'a>(
        self,
        backend: &'a dyn traits::Backend,
    ) -> ExposureFileRefs<'a> {
        self.0
            .into_iter()
            .map(|v| v.bind(backend))
            .collect::<Vec<_>>()
            .into()
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

impl ExposureFileViews {
    pub(super) fn bind<'a>(
        self,
        backend: &'a dyn traits::Backend,
    ) -> ExposureFileViewRefs<'a> {
        self.0
            .into_iter()
            .map(|v| v.bind(backend))
            .collect::<Vec<_>>()
            .into()
    }
}

impl ExposureFileViewRef<'_> {
    pub fn into_inner(self) -> ExposureFileView {
        self.inner
    }
}
