use async_trait::async_trait;
use crate::{
    error::BackendError,
    profile::{
        Profile,
        ViewTaskTemplate,
        ViewTaskTemplates,
        ViewTaskTemplateProfile,
    },
};

#[async_trait]
pub trait ProfileBackend {
    async fn insert_profile(
        &self,
        title: &str,
        description: &str,
    ) -> Result<i64, BackendError>;
    async fn update_profile_by_fields(
        &self,
        id: i64,
        title: &str,
        description: &str,
    ) -> Result<bool, BackendError>;
    async fn select_profile_by_id(
        &self,
        id: i64,
    ) -> Result<Profile, BackendError>;
    // TODO listing/query for set of profiles.
    // This may be implemented at the backends for the linked types.
}

#[async_trait]
pub trait ViewTaskTemplateBackend {
    async fn insert_view_task_template(
        &self,
        view_key: &str,
        description: &str,
        task_template_id: i64,
    ) -> Result<i64, BackendError>;
    async fn update_view_task_template_by_fields(
        &self,
        id: i64,
        view_key: &str,
        description: &str,
        task_template_id: i64,
    ) -> Result<bool, BackendError>;
    async fn select_view_task_template_by_id(
        &self,
        id: i64,
    ) -> Result<ViewTaskTemplate, BackendError>;
}

#[async_trait]
pub trait ProfileViewsBackend {
    // TODO determine if exposing these low level records are necessary.
    async fn insert_profile_views(
        &self,
        profile_id: i64,
        view_task_template_id: i64,
    ) -> Result<i64, BackendError>;
    async fn delete_profile_views(
        &self,
        profile_id: i64,
        view_task_template_id: i64,
    ) -> Result<bool, BackendError>;
    // this returns the records external to the table that this trait
    // suppposedly manages.
    async fn get_view_task_templates_for_profile(
        &self,
        profile_id: i64,
    ) -> Result<ViewTaskTemplates, BackendError>;
}

#[async_trait]
pub trait ViewTaskTemplateProfileBackend: ProfileBackend
    + ProfileViewsBackend

    + Sync
{
    async fn get_view_task_template_profile(
        &self,
        profile_id: i64,
    ) -> Result<ViewTaskTemplateProfile, BackendError> {
        let profile = ProfileBackend::select_profile_by_id(
            self,
            profile_id,
        ).await?;
        let view_task_templates = ProfileViewsBackend::get_view_task_templates_for_profile(
            self,
            profile_id,
        ).await?;
        Ok(ViewTaskTemplateProfile {
            profile,
            view_task_templates,
        })
    }
}
