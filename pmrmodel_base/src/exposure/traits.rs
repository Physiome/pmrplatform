use async_trait::async_trait;
use crate::{
    error::BackendError,
    exposure::{
        Exposure,
        Exposures,
        ExposureFile,
        ExposureFiles,
    },
};

#[async_trait]
pub trait ExposureBackend {
    async fn insert(
        &self,
        workspace_id: i64,
        workspace_tag_id: Option<i64>,
        commit_id: &str,
        root_exposure_file_id: Option<i64>,
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
