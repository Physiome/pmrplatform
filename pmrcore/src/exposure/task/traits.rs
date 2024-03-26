use async_trait::async_trait;
use crate::{
    error::BackendError,
    exposure::task::ExposureFileViewTask,
    profile::{
        ViewTaskTemplate,
        ViewTaskTemplateProfile,
    },
};

#[async_trait]
pub trait ExposureTaskTemplateBackend {
    async fn set_file_templates(
        &self,
        exposure_file_id: i64,
        task_template_ids: impl Iterator<Item = i64> + Send,
    ) -> Result<(), BackendError>;
    async fn get_file_templates(
        &self,
        exposure_file_id: i64,
    ) -> Result<Vec<ViewTaskTemplate>, BackendError>;

    async fn set_vtt_profile(
        &self,
        exposure_file_id: i64,
        profile: ViewTaskTemplateProfile,
    ) -> Result<(), BackendError> {
        self.set_file_templates(
            exposure_file_id,
            profile.view_task_templates
                .iter()
                .map(|vtt| vtt.id),
        ).await
    }
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
}
