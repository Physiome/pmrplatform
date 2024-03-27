use async_trait::async_trait;

use pmrcore::{
    error::BackendError,
    exposure::profile::{
        ExposureFileProfile,
        traits::ExposureFileProfileBackend,
    },
};

use crate::backend::db::SqliteBackend;

#[async_trait]
impl ExposureFileProfileBackend for SqliteBackend {
    async fn set_ef_profile(
        &self,
        exposure_file_id: i64,
        profile_id: i64,
    ) -> Result<(), BackendError> {
        todo!()
    }

    async fn get_ef_profile(
        &self,
        exposure_file_id: i64,
    ) -> Result<ExposureFileProfile, BackendError> {
        todo!()
    }
}
