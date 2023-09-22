use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    exposure::task::traits::ExposureTaskBackend,
};

use crate::{
    backend::db::SqliteBackend,
};

#[async_trait]
impl ExposureTaskBackend for SqliteBackend {
    async fn set_file_templates(
        &self,
        exposure_file_id: i64,
        task_template_ids: impl Iterator<Item = i64> + Send,
    ) -> Result<(), BackendError> {
        todo!()
    }
}
