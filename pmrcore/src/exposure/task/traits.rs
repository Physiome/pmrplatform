use async_trait::async_trait;
use crate::{
    error::{
        BackendError,
        Error,
    },
    exposure::task::ExposureFileViewTask,
    profile::ViewTaskTemplate,
};

#[async_trait]
pub trait ExposureTaskTemplateBackend {
    async fn set_file_templates(
        &self,
        exposure_file_id: i64,
        task_template_ids: &[i64],
    ) -> Result<(), BackendError>;
    async fn get_file_templates(
        &self,
        exposure_file_id: i64,
    ) -> Result<Vec<ViewTaskTemplate>, BackendError>;

}

#[async_trait]
pub trait ExposureTaskBackend {
    async fn create_task_for_view(
        &self,
        exposure_file_view_id: i64,
        view_task_template_id: i64,
        task_id: Option<i64>,
    ) -> Result<i64, BackendError>;
    async fn select_task_for_view(
        &self,
        exposure_file_id: i64,
    ) -> Result<Option<ExposureFileViewTask>, BackendError>;
    async fn finalize_task_id(
        &self,
        task_id: i64,
    ) -> Result<Option<(i64, Option<String>)>, Error>;
}
