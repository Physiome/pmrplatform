use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    profile::ViewTaskTemplateProfile,
    profile::traits::ViewTaskTemplateProfileBackend,
};

use crate::{
    backend::db::SqliteBackend,
};

#[async_trait]
impl ViewTaskTemplateProfileBackend for SqliteBackend {
    async fn get_view_task_template_profile(
        &self,
        profile_id: i64,
    ) -> Result<ViewTaskTemplateProfile, BackendError> {
        todo!()
    }
}
