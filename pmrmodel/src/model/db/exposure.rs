use async_trait::async_trait;
use pmrmodel_base::exposure::{
    Exposure,
    Exposures,
};

#[async_trait]
pub trait ExposureBackend {
    async fn add_exposure(
        &self,
        workspace_id: i64,
        workspace_tag_id: Option<i64>,
        commit_id: String,
        root_exposure_file_id: Option<i64>,
    ) -> Result<i64, sqlx::Error>;
    async fn list_exposures_for_workspace(
        &self,
        workspace_id: i64,
    ) -> Result<Exposures, sqlx::Error>;
    async fn get_exposure_by_id(
        &self,
        id: i64,
    ) -> Result<Exposure, sqlx::Error>;
}
