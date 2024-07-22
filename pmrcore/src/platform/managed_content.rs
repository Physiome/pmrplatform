use async_trait::async_trait;
use crate::{
    error::BackendError,
    exposure::{
        self,
        profile::traits::ExposureFileProfileBackend,
        task::{
            traits::{
                ExposureTaskBackend,
                ExposureTaskTemplateBackend,
            },
        },
        traits::{
            ExposureBackend,
            ExposureFileBackend,
            ExposureFileViewBackend,
        },
    },
    platform::PlatformUrl,
    profile::{
        ViewTaskTemplateProfile,
        traits::{
            ProfileBackend,
            ViewTaskTemplateBackend,
            ProfileViewsBackend,
            ViewTaskTemplateProfileBackend,
        },
    },
    workspace,
    workspace::traits::{
        WorkspaceBackend,
        WorkspaceAliasBackend,
        WorkspaceSyncBackend,
        WorkspaceTagBackend,
    },
};

/// MCPlatform - Managed Content Platform
///
/// This platform is used to manage the core contents relating to PMR,
/// i.e. workspace and exposures.  It is applicable to everything that
/// correctly implements the relevant backends that compose this trait.
#[async_trait]
pub trait MCPlatform: WorkspaceBackend
    + WorkspaceAliasBackend
    + WorkspaceSyncBackend
    + WorkspaceTagBackend
    + ExposureBackend
    + ExposureFileBackend
    + ExposureFileProfileBackend
    + ExposureFileViewBackend
    + ExposureTaskBackend
    + ExposureTaskTemplateBackend

    + ProfileBackend
    + ViewTaskTemplateBackend
    + ProfileViewsBackend
    + ViewTaskTemplateProfileBackend

    + PlatformUrl
{
    /// get the `ExposureRef` by the provided `id`
    async fn get_exposure<'a>(
        &'a self,
        id: i64,
    ) -> Result<exposure::ExposureRef<'a, Self>, BackendError>
        where Self: Sized
    {
        ExposureBackend::get_id(self, id)
            .await
            .map(|v| v.bind(self))
    }

    /// get all `ExposureRefs` for the provided `workspace_id`
    async fn get_exposures<'a>(
        &'a self,
        workspace_id: i64,
    ) -> Result<exposure::ExposureRefs<'a, Self>, BackendError>
        where Self: Sized
    {
        ExposureBackend::list_for_workspace(self, workspace_id)
            .await
            .map(|v| v.bind(self).into())
    }

    /// get the `ExposureFileRef` by the provided `id`
    async fn get_exposure_file<'a>(
        &'a self,
        id: i64,
    ) -> Result<exposure::ExposureFileRef<'a, Self>, BackendError>
        where Self: Sized
    {
        ExposureFileBackend::get_id(self, id)
            .await
            .map(|v| v.bind(self))
    }

    /// get the `ExposureFileRef` by the provided `exposure_id` and `workspace_file_path`
    async fn get_exposure_file_by_id_path<'a>(
        &'a self,
        exposure_id: i64,
        workspace_file_path: &str,
    ) -> Result<exposure::ExposureFileRef<'a, Self>, BackendError>
        where Self: Sized
    {
        ExposureFileBackend::get_by_exposure_filepath(
            self,
            exposure_id,
            workspace_file_path,
        )
            .await
            .map(|v| v.bind(self))
    }

    /// get all `ExposureFileRefs` for the provided `exposure_id`
    async fn get_exposure_files<'a>(
        &'a self,
        exposure_id: i64,
    ) -> Result<exposure::ExposureFileRefs<'a, Self>, BackendError>
        where Self: Sized
    {
        ExposureFileBackend::list_for_exposure(self, exposure_id)
            .await
            .map(|v| v.bind(self).into())
    }

    /// get the `ExposureFileViewRef` by the provided `id`
    async fn get_exposure_file_view<'a>(
        &'a self,
        id: i64,
    ) -> Result<exposure::ExposureFileViewRef<'a, Self>, BackendError>
        where Self: Sized
    {
        ExposureFileViewBackend::get_id(self, id)
            .await
            .map(|v| v.bind(self))
    }

    /// get the `ExposureFileViewRef` by the provided `exposure_file_id`
    /// and the `view_task_template_id`.
    async fn get_exposure_file_view_by_file_template<'a>(
        &'a self,
        exposure_file_id: i64,
        view_task_template_id: i64,
    ) -> Result<exposure::ExposureFileViewRef<'a, Self>, BackendError>
        where Self: Sized
    {
        ExposureFileViewBackend::get_by_file_view_template(
                self,
                exposure_file_id,
                view_task_template_id,
            )
            .await
            .map(|v| v.bind(self))
    }

    /// get all `ExposureFileViewRefs` for the provided `exposure_file_id`
    async fn get_exposure_file_views<'a>(
        &'a self,
        exposure_file_id: i64,
    ) -> Result<exposure::ExposureFileViewRefs<'a, Self>, BackendError>
        where Self: Sized
    {
        ExposureFileViewBackend::list_for_exposure_file(self, exposure_file_id)
            .await
            .map(|v| v.bind(self))
    }

    /// get the `WorkspaceRef` by the provided `id`
    async fn get_workspace<'a>(
        &'a self,
        id: i64,
    ) -> Result<workspace::WorkspaceRef<'a, Self>, BackendError>
        where Self: Sized
    {
        // WorkspaceBackend::get_id(self, id)
        workspace::traits::WorkspaceBackend::get_workspace_by_id(self, id)
            .await
            .map(|v| v.bind(self))
    }

    /// get the `WorkspaceRef` by the provided `id`
    async fn list_workspaces<'a>(
        &'a self,
    ) -> Result<workspace::WorkspaceRefs<'a, Self>, BackendError>
        where Self: Sized
    {
        workspace::traits::WorkspaceBackend::list_workspaces(self)
            .await
            .map(|v| v.bind(self).into())
    }

    async fn set_ef_vttprofile(
        &self,
        exposure_file_id: i64,
        vttp: ViewTaskTemplateProfile,
    ) -> Result<(), BackendError> {
        ExposureTaskTemplateBackend::set_file_templates(
            self,
            exposure_file_id,
            vttp.view_task_templates
                .iter()
                .map(|vtt| vtt.id)
                .collect::<Vec<_>>()
                .as_slice(),
        ).await?;
        ExposureFileProfileBackend::set_ef_profile(
            self,
            exposure_file_id,
            vttp.profile.id,
        ).await?;
        Ok(())
    }
}

impl<P: workspace::traits::WorkspaceBackend
    + WorkspaceAliasBackend
    + WorkspaceSyncBackend
    + WorkspaceTagBackend
    + ExposureBackend
    + ExposureFileBackend
    + ExposureFileProfileBackend
    + ExposureFileViewBackend
    + ExposureTaskBackend
    + ExposureTaskTemplateBackend

    + ProfileBackend
    + ViewTaskTemplateBackend
    + ProfileViewsBackend
    + ViewTaskTemplateProfileBackend

    + PlatformUrl
> MCPlatform for P {}
