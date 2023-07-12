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
    /// Inserts a new `Exposure` entry.
    ///
    /// Returns the id of the inserted entry.
    async fn insert(
        &self,
        workspace_id: i64,
        workspace_tag_id: Option<i64>,
        commit_id: &str,
        default_file_id: Option<i64>,
    ) -> Result<i64, BackendError>;

    /// Returns all `Exposures` for the given `workspace_id`.
    async fn list_for_workspace(
        &self,
        workspace_id: i64,
    ) -> Result<Exposures, BackendError>;

    /// Returns the `Exposure` for the given `id`.
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<Exposure, BackendError>;

    /// For the given `Exposure` identified by its `id`, set the default
    /// `ExposureFile` via its `id`.
    async fn set_default_file(
        &self,
        id: i64,
        file_id: i64,
    ) -> Result<bool, BackendError>;
}

#[async_trait]
pub trait ExposureFileBackend {
    /// Inserts a new `ExposureFile` entry.
    ///
    /// Returns the id of the inserted entry.
    async fn insert(
        &self,
        exposure_id: i64,
        workspace_file_path: &str,
        default_view_id: Option<i64>,
    ) -> Result<i64, BackendError>;

    /// Returns all `ExposureFiles` for the given `exposure_id`.
    async fn list_for_exposure(
        &self,
        exposure_id: i64,
    ) -> Result<ExposureFiles, BackendError>;

    /// Returns the `ExposureFile` for the given `id`.
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<ExposureFile, BackendError>;

    /// For the given `ExposureFile` identified by its `id`, set the
    /// default `ExposureFileView` via its `id`.
    async fn set_default_view(
        &self,
        id: i64,
        file_id: i64,
    ) -> Result<bool, BackendError>;
}

#[async_trait]
pub trait ExposureFileViewBackend {
    /// Inserts a new `ExposureFileView` entry.
    ///
    /// Returns the id of the inserted entry.
    async fn insert(
        &self,
        exposure_file_id: i64,
        view_key: &str,
    ) -> Result<i64, BackendError>;

    /// Returns all `ExposureFileViews` for the given `exposure_file_id`.
    async fn list_for_exposure_file(
        &self,
        exposure_file_id: i64,
    ) -> Result<ExposureFileViews, BackendError>;

    /// Returns the `ExposureFileView` for the given `id`.
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<ExposureFileView, BackendError>;
}
