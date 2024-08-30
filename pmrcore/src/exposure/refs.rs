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

pub struct ExposureRef<'a> {
    pub(super) inner: Exposure,
    pub(super) files: OnceLock<ExposureFileRefs<'a>>,
    pub(super) platform: &'a dyn MCPlatform,
    pub(super) parent: OnceLock<WorkspaceRef<'a>>,
}

pub struct ExposureRefs<'a>(pub(super) Vec<ExposureRef<'a>>);

pub struct ExposureFileRef<'a> {
    pub(super) inner: ExposureFile,
    pub(super) views: OnceLock<ExposureFileViewRefs<'a>>,
    pub(super) platform: &'a dyn MCPlatform,
    pub(super) parent: OnceLock<ExposureRef<'a>>,
}

pub struct ExposureFileRefs<'a>(pub(super) Vec<ExposureFileRef<'a>>);

pub struct ExposureFileViewRef<'a> {
    pub(super) inner: ExposureFileView,
    pub(super) platform: &'a dyn MCPlatform,
    pub(super) parent: OnceLock<ExposureFileRef<'a>>,
}

pub struct ExposureFileViewRefs<'a>(pub(super) Vec<ExposureFileViewRef<'a>>);

impl Exposure {
    pub(crate) fn bind<'a>(
        self,
        platform: &'a dyn MCPlatform,
    ) -> ExposureRef<'a> {
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
    pub(crate) fn bind<'a>(
        self,
        platform: &'a dyn MCPlatform,
    ) -> ExposureRefs<'a> {
        self.0
            .into_iter()
            .map(|v| v.bind(platform))
            .collect::<Vec<_>>()
            .into()
    }
}

impl ExposureRef<'_> {
    pub fn into_inner(self) -> Exposure {
        self.inner
    }

    pub fn clone_inner(&self) -> Exposure {
        self.inner.clone()
    }
}

impl<'a> ExposureRef<'a> {
    pub async fn get_file(
        &self,
        path: &str,
    ) -> Result<ExposureFileRef<'a>, BackendError> {
        Ok(
            self.platform.get_by_exposure_filepath(
                self.inner.id,
                path,
            )
                .await?
                .bind(self.platform)
        )
    }
}

impl ExposureFile {
    pub(crate) fn bind<'a>(
        self,
        platform: &'a dyn MCPlatform,
    ) -> ExposureFileRef<'a> {
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
    pub(crate) fn bind<'a>(
        self,
        platform: &'a dyn MCPlatform,
    ) -> ExposureFileRefs<'a,> {
        self.0
            .into_iter()
            .map(|v| v.bind(platform))
            .collect::<Vec<_>>()
            .into()
    }
}

impl ExposureFileRef<'_> {
    pub fn into_inner(self) -> ExposureFile {
        self.inner
    }
    pub fn clone_inner(&self) -> ExposureFile {
        let mut inner = self.inner.clone();
        inner.views = self.views.get().map(|v| v
            .iter()
            .map(|v| v.clone_inner())
            .collect::<Vec<_>>()
            .into()
        );
        inner
    }
}

impl ExposureFileView {
    pub(crate) fn bind<'a>(
        self,
        platform: &'a dyn MCPlatform,
    ) -> ExposureFileViewRef<'a> {
        ExposureFileViewRef {
            inner: self,
            platform: platform,
            parent: OnceLock::new(),
        }
    }
}

impl ExposureFileViews {
    pub(crate) fn bind<'a>(
        self,
        platform: &'a dyn MCPlatform,
    ) -> ExposureFileViewRefs<'a> {
        self.0
            .into_iter()
            .map(|v| v.bind(platform))
            .collect::<Vec<_>>()
            .into()
    }
}

impl<'a> ExposureFileViewRef<'a> {
    pub async fn update_view_key(
        &mut self,
        view_key: Option<&'a str>,
    ) -> Result<bool, BackendError> {
        let result = ExposureFileViewBackend::update_view_key(
            self.platform,
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
        let result = ExposureFileViewBackend::update_exposure_file_view_task_id(
            self.platform,
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

impl ExposureFileViewRef<'_> {
    pub fn into_inner(self) -> ExposureFileView {
        self.inner
    }
    pub fn clone_inner(&self) -> ExposureFileView {
        self.inner.clone()
    }
}
