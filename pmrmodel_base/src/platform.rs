use async_trait::async_trait;
use crate::{
    error::BackendError,
    exposure,
    exposure::traits::{
        ExposureBackend,
        ExposureFileBackend,
        ExposureFileViewBackend,
    },
    workspace,
    workspace::traits::{
        WorkspaceBackend,
        WorkspaceAliasBackend,
        WorkspaceSyncBackend,
        WorkspaceTagBackend,
    },
};

#[async_trait]
pub trait Platform: WorkspaceBackend
    + WorkspaceAliasBackend
    + WorkspaceSyncBackend
    + WorkspaceTagBackend
    + ExposureBackend
    + ExposureFileBackend
    + ExposureFileViewBackend
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
}

impl<B: workspace::traits::WorkspaceBackend
    + WorkspaceAliasBackend
    + WorkspaceSyncBackend
    + WorkspaceTagBackend
    + ExposureBackend
    + ExposureFileBackend
    + ExposureFileViewBackend
> Platform for B {}
