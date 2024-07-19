use std::sync::OnceLock;
use crate::error::BackendError;
use crate::exposure::{
    Exposure,
    Exposures,
    ExposureFile,
    ExposureFiles,
    ExposureFileView,
    ExposureFileViews,
};
use crate::exposure::traits::ExposureFileViewBackend;
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

    pub fn clone_inner(&self) -> Exposure {
        self.inner.clone()
    }
}

impl<'a, P: MCPlatform + Sized> ExposureRef<'a, P> {
    pub async fn get_file(
        &self,
        path: &str,
    ) -> Result<ExposureFileRef<'a, P>, BackendError> {
        Ok(
            self.platform.get_by_exposure_filepath(
                self.inner.id,
                path,
            )
                .await?
                .bind(&self.platform)
        )
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
    pub fn clone_inner(&self) -> ExposureFile {
        self.inner.clone()
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

impl<'a, P: MCPlatform + Sized> ExposureFileViewRef<'a, P> {
    pub async fn update_view_key(
        &mut self,
        view_key: Option<&'a str>,
    ) -> Result<bool, BackendError> {
        let backend: &dyn ExposureFileViewBackend = self.platform;
        let result = backend.update_view_key(
            self.inner.id,
            view_key,
        ).await?;
        if !result {
            return Err(BackendError::AppInvariantViolation(
                format!(
                    "Underlying ExposureFileView is gone (id: {})",
                    self.inner.id
                )
            ))
        }
        self.inner.view_key = view_key.map(|s| s.to_string());
        Ok(result)
    }

    pub async fn update_exposure_file_view_task_id(
        &mut self,
        exposure_file_view_task_id: Option<i64>,
    ) -> Result<bool, BackendError> {
        let backend: &dyn ExposureFileViewBackend = self.platform;
        let result = backend.update_exposure_file_view_task_id(
            self.inner.id,
            exposure_file_view_task_id,
        ).await?;
        if !result {
            return Err(BackendError::AppInvariantViolation(
                format!(
                    "Underlying ExposureFileView is gone (id: {})",
                    self.inner.id
                )
            ))
        }
        self.inner.exposure_file_view_task_id = exposure_file_view_task_id;
        Ok(result)
    }
}

impl<P: MCPlatform + Sized> ExposureFileViewRef<'_, P> {
    pub fn into_inner(self) -> ExposureFileView {
        self.inner
    }
    pub fn clone_inner(&self) -> ExposureFileView {
        self.inner.clone()
    }
}
