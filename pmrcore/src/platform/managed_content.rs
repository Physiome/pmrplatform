use std::collections::BTreeMap;
use async_trait::async_trait;
use crate::{
    alias::{
        traits::AliasBackend,
        AliasEntries,
        AliasEntry,
    },
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
        Exposure,
    },
    idgen::traits::GenAliasBackend,
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
    workspace::{
        WorkspaceRef,
        self,
        traits::{
            Workspace as _,
            WorkspaceBackend,
            WorkspaceSyncBackend,
            WorkspaceTagBackend,
        },
    },
};

/// MCPlatform - Managed Content Platform
///
/// This platform is used to manage the core contents relating to PMR,
/// i.e. workspace and exposures.  It is applicable to everything that
/// correctly implements the relevant backends that compose this trait.
#[async_trait]
pub trait MCPlatform: WorkspaceBackend
    + WorkspaceSyncBackend
    + WorkspaceTagBackend
    + ExposureBackend
    + ExposureFileBackend
    + ExposureFileProfileBackend
    + ExposureFileViewBackend
    + ExposureTaskBackend
    + ExposureTaskTemplateBackend

    + AliasBackend
    + GenAliasBackend

    + ProfileBackend
    + ViewTaskTemplateBackend
    + ProfileViewsBackend
    + ViewTaskTemplateProfileBackend

    + PlatformUrl

    + Send
    + Sync
{
    fn as_dyn(&self) -> &dyn MCPlatform;

    /// create workspace with a default alias
    async fn create_aliased_workspace<'a>(
        &'a self,
        url: &str,
        description: &str,
        long_description: &str,
    ) -> Result<AliasEntry<WorkspaceRef<'a>>, BackendError> {
        let id = WorkspaceBackend::add_workspace(self, url, description, long_description).await?;
        let alias = GenAliasBackend::next(self).await?.to_string();
        self.add_alias("workspace", id, &alias).await?;
        let entity = self.get_workspace(id).await?;
        Ok(AliasEntry { alias, entity })
    }

    /// get the `ExposureRef` by the provided `id`
    async fn get_exposure<'a>(
        &'a self,
        id: i64,
    ) -> Result<exposure::ExposureRef<'a>, BackendError> {
        ExposureBackend::get_id(self, id)
            .await
            .map(|v| v.bind(self.as_dyn()))
    }

    /// get all `ExposureRefs` for the provided `workspace_id`
    async fn get_exposures<'a>(
        &'a self,
        workspace_id: i64,
    ) -> Result<exposure::ExposureRefs<'a>, BackendError> {
        ExposureBackend::list_for_workspace(self, workspace_id)
            .await
            .map(|v| v.bind(self.as_dyn()).into())
    }

    /// get the `ExposureFileRef` by the provided `id`
    async fn get_exposure_file<'a>(
        &'a self,
        id: i64,
    ) -> Result<exposure::ExposureFileRef<'a>, BackendError> {
        ExposureFileBackend::get_id(self, id)
            .await
            .map(|v| v.bind(self.as_dyn()))
    }

    /// get the `ExposureFileRef` by the provided `exposure_id` and `workspace_file_path`
    async fn get_exposure_file_by_id_path<'a>(
        &'a self,
        exposure_id: i64,
        workspace_file_path: &str,
    ) -> Result<exposure::ExposureFileRef<'a>, BackendError> {
        ExposureFileBackend::get_by_exposure_filepath(
            self,
            exposure_id,
            workspace_file_path,
        )
            .await
            .map(|v| v.bind(self.as_dyn()))
    }

    /// get all `ExposureFileRefs` for the provided `exposure_id`
    async fn get_exposure_files<'a>(
        &'a self,
        exposure_id: i64,
    ) -> Result<exposure::ExposureFileRefs<'a>, BackendError> {
        ExposureFileBackend::list_for_exposure(self, exposure_id)
            .await
            .map(|v| v.bind(self.as_dyn()).into())
    }

    /// get the `ExposureFileViewRef` by the provided `id`
    async fn get_exposure_file_view<'a>(
        &'a self,
        id: i64,
    ) -> Result<exposure::ExposureFileViewRef<'a>, BackendError> {
        ExposureFileViewBackend::get_id(self, id)
            .await
            .map(|v| v.bind(self.as_dyn()))
    }

    /// get the `ExposureFileViewRef` by the provided `exposure_file_id`
    /// and the `view_task_template_id`.
    async fn get_exposure_file_view_by_file_template<'a>(
        &'a self,
        exposure_file_id: i64,
        view_task_template_id: i64,
    ) -> Result<exposure::ExposureFileViewRef<'a>, BackendError> {
        ExposureFileViewBackend::get_by_file_view_template(
                self,
                exposure_file_id,
                view_task_template_id,
            )
            .await
            .map(|v| v.bind(self.as_dyn()))
    }

    /// get the `ExposureFileViewRef` by the provided `exposure_file_id`
    /// and the `view_key`.
    async fn get_exposure_file_view_by_file_view_key<'a>(
        &'a self,
        exposure_file_id: i64,
        view_key: &str,
    ) -> Result<exposure::ExposureFileViewRef<'a>, BackendError> {
        ExposureFileViewBackend::get_by_file_view_key(
                self,
                exposure_file_id,
                view_key,
            )
            .await
            .map(|v| v.bind(self.as_dyn()))
    }

    /// get all `ExposureFileViewRefs` for the provided `exposure_file_id`
    async fn get_exposure_file_views<'a>(
        &'a self,
        exposure_file_id: i64,
    ) -> Result<exposure::ExposureFileViewRefs<'a>, BackendError> {
        ExposureFileViewBackend::list_for_exposure_file(self, exposure_file_id)
            .await
            .map(|v| v.bind(self.as_dyn()))
    }

    /// get the `WorkspaceRef` by the provided `id`
    async fn get_workspace<'a>(
        &'a self,
        id: i64,
    ) -> Result<workspace::WorkspaceRef<'a>, BackendError> {
        // WorkspaceBackend::get_id(self, id)
        workspace::traits::WorkspaceBackend::get_workspace_by_id(self, id)
            .await
            .map(|v| v.bind(self.as_dyn()))
    }

    /// get the `WorkspaceRef` by the provided `id`
    async fn list_workspaces<'a>(
        &'a self,
    ) -> Result<workspace::WorkspaceRefs<'a>, BackendError> {
        workspace::traits::WorkspaceBackend::list_workspaces(self)
            .await
            .map(|v| v.bind(self.as_dyn()).into())
    }

    /// list workspace by their aliases.
    ///
    /// This is provided as a generic implementation
    async fn list_aliased_workspaces<'a>(
        &'a self,
    ) -> Result<AliasEntries<WorkspaceRef<'a>>, BackendError> {
        let this = self.as_dyn();
        let aliases = self.aliases_by_kind("workspace").await?;
        let mut id_map = aliases.into_iter()
            .map(|(key, id)| (id, key))
            .collect::<BTreeMap<_, _>>();
        let ids = id_map.keys()
            .map(|id| *id)
            .collect::<Vec<_>>();
        Ok(AliasEntries {
            kind: "workspace".to_string(),
            entries: self.list_workspace_by_ids(&ids).await?
                .into_iter()
                .map(|workspace| AliasEntry {
                    alias: id_map.remove(&workspace.id())
                        .expect("unexpected id queried without an alias queried"),
                    entity: workspace.bind(this),
                })
                .collect::<Vec<_>>(),
        })
    }

    /// get the `ExposureRefs`
    async fn list_exposures<'a>(
        &'a self,
    ) -> Result<exposure::ExposureRefs<'a>, BackendError> {
        ExposureBackend::list(self)
            .await
            .map(|v| v.bind(self.as_dyn()).into())
    }

    /// get `ExposureRefs` for the workspace by the `workspace_id`
    async fn list_exposures_for_workspace<'a>(
        &'a self,
        workspace_id: i64,
    ) -> Result<exposure::ExposureRefs<'a>, BackendError> {
        ExposureBackend::list_for_workspace(self, workspace_id)
            .await
            .map(|v| v.bind(self.as_dyn()).into())
    }

    /// list exposures by their aliases.
    ///
    /// This is provided as a generic implementation
    async fn list_aliased_exposures<'a>(
        &'a self,
    ) -> Result<AliasEntries<exposure::ExposureRef<'a>>, BackendError> {
        let this = self.as_dyn();
        let aliases = self.aliases_by_kind("exposure").await?;
        let mut id_map = aliases.into_iter()
            .map(|(key, id)| (id, key))
            .collect::<BTreeMap<_, _>>();
        let ids = id_map.keys()
            .map(|id| *id)
            .collect::<Vec<_>>();
        Ok(AliasEntries {
            kind: "exposure".to_string(),
            entries: ExposureBackend::list_by_ids(self, &ids).await?
                .into_iter()
                .map(|exposure| AliasEntry {
                    alias: id_map.remove(&exposure.id)
                        .expect("unexpected id queried without an alias queried"),
                    entity: exposure.bind(this),
                })
                .collect::<Vec<_>>(),
        })
    }

    /// list `ExposureRefs` for the workspace by the `workspace_id` with their alias
    ///
    /// This is provided as a generic implementation.
    async fn list_aliased_exposures_for_workspace<'a>(
        &'a self,
        workspace_id: i64,
    ) -> Result<AliasEntries<exposure::ExposureRef<'a>>, BackendError> {
        let this = self.as_dyn();
        let exposures = ExposureBackend::list_for_workspace(self, workspace_id)
            .await?;
        let mut id_map = exposures.into_iter()
            .map(|exposure| (exposure.id, exposure))
            .collect::<BTreeMap<_, _>>();
        let ids = id_map.keys()
            .map(|id| *id)
            .collect::<Vec<_>>();

        Ok(AliasEntries {
            kind: "exposure".to_string(),
            entries: self.aliases_by_kind_ids("exposure", &ids).await?
                .into_iter()
                .map(|(alias, id)| AliasEntry {
                    alias,
                    entity: id_map.remove(&id)
                        .expect("unexpected id queried for alias")
                        .bind(this),
                })
                .collect::<Vec<_>>()
        })
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

pub trait DefaultMCPlatform: MCPlatform {}

impl<P: workspace::traits::WorkspaceBackend
    + WorkspaceSyncBackend
    + WorkspaceTagBackend
    + ExposureBackend
    + ExposureFileBackend
    + ExposureFileProfileBackend
    + ExposureFileViewBackend
    + ExposureTaskBackend
    + ExposureTaskTemplateBackend

    + AliasBackend
    + GenAliasBackend

    + ProfileBackend
    + ViewTaskTemplateBackend
    + ProfileViewsBackend
    + ViewTaskTemplateProfileBackend

    + PlatformUrl

    + DefaultMCPlatform

    + Send
    + Sync
> MCPlatform for P {
    fn as_dyn(&self) -> &dyn MCPlatform {
        self
    }
}
