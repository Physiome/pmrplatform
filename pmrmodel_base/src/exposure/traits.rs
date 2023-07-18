use async_trait::async_trait;
use std::ops::Deref;
use crate::{
    error::{
        BackendError,
        ValueError,
    },
    exposure,
};

pub trait Exposure<'a, S> {
    fn id(&self) -> i64;
    fn workspace_id(&self) -> i64;
    fn workspace_tag_id(&self) -> Option<i64>;
    fn commit_id(&self) -> &str;
    fn created_ts(&self) -> i64;
    fn default_file_id(&self) -> Option<i64>;
    fn files(&'a self) -> Result<&'a S, ValueError>;
}

pub trait ExposureFile<'a, S> {
    fn id(&self) -> i64;
    fn exposure_id(&self) -> i64;
    fn workspace_file_path(&self) -> &str;
    fn default_view_id(&self) -> Option<i64>;
    fn views(&'a self) -> Result<&'a S, ValueError>;
}

pub trait ExposureFileView {
    fn id(&self) -> i64;
    fn exposure_file_id(&self) -> i64;
    fn view_key(&self) -> &str;
}

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
    ) -> Result<exposure::Exposures, BackendError>;

    /// Returns the `Exposure` for the given `id`.
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<exposure::Exposure, BackendError>;

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
    ) -> Result<exposure::ExposureFiles, BackendError>;

    /// Returns the `ExposureFile` for the given `id`.
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<exposure::ExposureFile, BackendError>;

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
    ) -> Result<exposure::ExposureFileViews, BackendError>;

    /// Returns the `ExposureFileView` for the given `id`.
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<exposure::ExposureFileView, BackendError>;
}

#[async_trait]
pub trait Backend: ExposureBackend + ExposureFileBackend + ExposureFileViewBackend {
    /// get the `ExposureRef` by the provided `id`
    async fn get_exposure<'a>(
        &'a self,
        id: i64,
    ) -> Result<exposure::ExposureRef<'a>, BackendError>
        where Self: Sized
    {
        ExposureBackend::get_id(self, id)
            .await
            .map(|v| v.bind(self))
    }

    /// get all `ExposureRefs` for the provided `workspace_id`
    async fn get_exposures<'a>(
        &'a self,
        workspace_id: i64,
    ) -> Result<exposure::ExposureRefs<'a>, BackendError>
        where Self: Sized
    {
        ExposureBackend::list_for_workspace(self, workspace_id)
            .await
            .map(|v| v.bind(self).into())
    }

    /// get the `ExposureFileRef` by the provided `id`
    async fn get_exposure_file<'a>(
        &'a self,
        id: i64,
    ) -> Result<exposure::ExposureFileRef<'a>, BackendError>
        where Self: Sized
    {
        ExposureFileBackend::get_id(self, id).
            await
            .map(|v| v.bind(self))
    }

    /// get all `ExposureFileRefs` for the provided `exposure_id`
    async fn get_exposure_files<'a>(
        &'a self,
        exposure_id: i64,
    ) -> Result<exposure::ExposureFileRefs<'a>, BackendError>
        where Self: Sized
    {
        ExposureFileBackend::list_for_exposure(self, exposure_id)
            .await
            .map(|v| v.bind(self).into())
    }

    /// get the `ExposureFileViewRef` by the provided `id`
    async fn get_exposure_file_view<'a>(
        &'a self,
        id: i64,
    ) -> Result<exposure::ExposureFileViewRef<'a>, BackendError>
        where Self: Sized
    {
        ExposureFileViewBackend::get_id(self, id)
            .await
            .map(|v| v.bind(self))
    }

    /// get all `ExposureFileViewRefs` for the provided `exposure_file_id`
    async fn get_exposure_file_views<'a>(
        &'a self,
        exposure_file_id: i64,
    ) -> Result<exposure::ExposureFileViewRefs<'a>, BackendError>
        where Self: Sized
    {
        ExposureFileViewBackend::list_for_exposure_file(self, exposure_file_id)
            .await
            .map(|v| v.bind(self))
    }
}
impl<B: ExposureBackend + ExposureFileBackend + ExposureFileViewBackend> Backend for B {}
