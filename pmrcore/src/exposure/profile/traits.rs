use async_trait::async_trait;

use crate::{
    error::BackendError,
    exposure::profile::ExposureFileProfile,
};

#[async_trait]
pub trait ExposureFileProfileBackend {
    async fn set_ef_profile(
        &self,
        exposure_file_id: i64,
        profile_id: i64,
    ) -> Result<(), BackendError>;
    async fn get_ef_profile(
        &self,
        exposure_file_id: i64,
    ) -> Result<ExposureFileProfile, BackendError>;
}
