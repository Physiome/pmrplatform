use async_trait::async_trait;

use crate::{
    error::BackendError,
    exposure::profile::ExposureFileProfile,
    task_template::UserInputMap,
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
    async fn update_ef_user_input(
        &self,
        exposure_file_id: i64,
        user_input: &UserInputMap,
    ) -> Result<(), BackendError>;
}
