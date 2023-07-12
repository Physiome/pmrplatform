use async_trait::async_trait;
use crate::{
    error::BackendError,
    exposure::{
        Exposure,
        Exposures,
        ExposureFile,
        ExposureFiles,
        ExposureFileView,
        ExposureFileViews,
    },
};

#[async_trait]
pub trait ExposureBackend {
    async fn insert(
        &self,
        workspace_id: i64,
        workspace_tag_id: Option<i64>,
        commit_id: &str,
        default_file_id: Option<i64>,
    ) -> Result<i64, BackendError>;
    async fn list_for_workspace(
        &self,
        workspace_id: i64,
    ) -> Result<Exposures, BackendError>;
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<Exposure, BackendError>;
}

#[async_trait]
pub trait ExposureFileBackend {
    async fn insert(
        &self,
        exposure_id: i64,
        workspace_file_path: &str,
        default_view_id: Option<i64>,
    ) -> Result<i64, BackendError>;
    async fn list_for_exposure(
        &self,
        exposure_id: i64,
    ) -> Result<ExposureFiles, BackendError>;
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<ExposureFile, BackendError>;
}

#[async_trait]
pub trait ExposureFileViewBackend {
    async fn insert(
        &self,
        exposure_file_id: i64,
        view_key: &str,
    ) -> Result<i64, BackendError>;
    async fn list_for_exposure_file(
        &self,
        exposure_file_id: i64,
    ) -> Result<ExposureFileViews, BackendError>;
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<ExposureFileView, BackendError>;
}
