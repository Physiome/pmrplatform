use futures::future;
use pmrcore::{
    profile::{
        Profile,
        ViewTaskTemplateProfile,
        traits::{
            ProfileBackend,
            ProfileViewsBackend,
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

    pub async fn add_view_task_template_profile(
        &self,
        vttp: ViewTaskTemplateProfile,
    ) -> Result<i64, PlatformError> {
        let profile_id = ProfileBackend::insert_profile(
            self.mc_platform.as_ref(),
            &vttp.profile.title,
            &vttp.profile.description,
        ).await?;
        for view_task_template in vttp.view_task_templates.into_iter() {
            let vtt_id = self.adds_view_task_template(view_task_template).await?;
            ProfileViewsBackend::insert_profile_views(
                self.mc_platform.as_ref(),
                profile_id,
                vtt_id,
            ).await?;
        }
        Ok(profile_id)
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
