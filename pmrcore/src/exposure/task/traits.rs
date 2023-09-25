use async_trait::async_trait;
use crate::{
    error::{
        BackendError,
        ValueError,
    },
    profile::ViewTaskTemplate,
};

#[async_trait]
pub trait ExposureTaskBackend {
    async fn set_file_templates(
        &self,
        exposure_file_id: i64,
        task_template_ids: impl Iterator<Item = i64> + Send,
    ) -> Result<(), BackendError>;
    async fn get_file_templates(
        &self,
        exposure_file_id: i64,
    ) -> Result<Vec<ViewTaskTemplate>, BackendError>;
}
