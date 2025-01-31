use futures::future;
use pmrcore::{
    profile::{
        Profile,
        ViewTaskTemplateProfile,
        traits::{
            ProfileBackend,
            ViewTaskTemplateProfileBackend,
        },
    },
    task_template::traits::TaskTemplateBackend,
};
use crate::{
    error::PlatformError,
    platform::Platform,
};

impl Platform {
    pub async fn list_profiles(
        &self,
    ) -> Result<Vec<Profile>, PlatformError> {
        Ok(ProfileBackend::list_profiles(
            self.mc_platform.as_ref(),
        ).await?)
    }

    pub async fn create_view_profile(
        &self,
    ) -> Result<(), PlatformError> {
        todo!()
    }

    pub async fn get_view_task_template_profile(
        &self,
        profile_id: i64,
    ) -> Result<ViewTaskTemplateProfile, PlatformError> {
        let mut result = ViewTaskTemplateProfileBackend::get_view_task_template_profile(
            self.mc_platform.as_ref(),
            profile_id,
        ).await?;
        future::try_join_all(result.view_task_templates.iter_mut().map(|vtt| async {
            Ok::<(), PlatformError>(vtt.task_template = Some(
                TaskTemplateBackend::get_task_template_by_id(
                    self.tm_platform.as_ref(),
                    vtt.task_template_id,
                ).await?
            ))
        })).await?;
        Ok(result)
    }
}

mod view_task_template;
