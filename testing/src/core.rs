use async_trait::async_trait;
use mockall::mock;
use pmrcore::{
    error::BackendError,
    exposure::{
        Exposure,
        Exposures,
        ExposureFile,
        ExposureFiles,
        ExposureFileView,
        ExposureFileViews,
        traits::{
            ExposureBackend,
            ExposureFileBackend,
            ExposureFileViewBackend,
        },
    },
    workspace::{
        Workspace,
        Workspaces,
        WorkspaceAlias,
        WorkspaceSync,
        WorkspaceSyncStatus,
        WorkspaceTag,
        traits::{
            WorkspaceAliasBackend,
            WorkspaceBackend,
            WorkspaceSyncBackend,
            WorkspaceTagBackend,
        },
    },
};

mock! {
    pub Platform {
        async fn exposure_insert(
            &self,
            workspace_id: i64,
            workspace_tag_id: Option<i64>,
            commit_id: &str,
            default_file_id: Option<i64>,
        ) -> Result<i64, BackendError>;
        async fn exposure_list_for_workspace(
            &self,
            workspace_id: i64,
        ) -> Result<Exposures, BackendError>;
        async fn exposure_get_id(
            &self,
            id: i64,
        ) -> Result<Exposure, BackendError>;
        async fn exposure_set_default_file(
            &self,
            id: i64,
            file_id: i64,
        ) -> Result<bool, BackendError>;

        async fn exposure_file_insert(
            &self,
            exposure_id: i64,
            workspace_file_path: &str,
            default_view_id: Option<i64>,
        ) -> Result<i64, BackendError>;
        async fn exposure_file_list_for_exposure(
            &self,
            exposure_id: i64,
        ) -> Result<ExposureFiles, BackendError>;
        async fn exposure_file_get_id(
            &self,
            id: i64,
        ) -> Result<ExposureFile, BackendError>;
        async fn exposure_file_set_default_view(
            &self,
            id: i64,
            file_id: i64,
        ) -> Result<bool, BackendError>;

        async fn exposure_file_view_insert(
            &self,
            exposure_file_id: i64,
            view_task_template_id: i64,
        ) -> Result<i64, BackendError>;
        async fn exposure_file_view_list_for_exposure_file(
            &self,
            exposure_file_id: i64,
        ) -> Result<ExposureFileViews, BackendError>;
        async fn exposure_file_view_get_id(
            &self,
            id: i64,
        ) -> Result<ExposureFileView, BackendError>;
        async fn exposure_file_view_update_view_key(
            &self,
            id: i64,
            view_key: &str,
        ) -> Result<bool, BackendError>;
    }

    #[async_trait]
    impl WorkspaceTagBackend for Platform {
        async fn index_workspace_tag(&self, workspace_id: i64, name: &str, commit_id: &str) -> Result<i64, BackendError>;
        async fn get_workspace_tags(&self, workspace_id: i64) -> Result<Vec<WorkspaceTag>, BackendError>;
    }

    #[async_trait]
    impl WorkspaceBackend for Platform {
        async fn add_workspace(
            &self, url: &str, description: &str, long_description: &str
        ) -> Result<i64, BackendError>;
        async fn update_workspace(
            &self, id: i64, description: &str, long_description: &str
        ) -> Result<bool, BackendError>;
        async fn list_workspaces(&self) -> Result<Workspaces, BackendError>;
        async fn get_workspace_by_id(&self, id: i64) -> Result<Workspace, BackendError>;
        async fn list_workspace_by_url(&self, url: &str) -> Result<Workspaces, BackendError>;
    }

    #[async_trait]
    impl WorkspaceSyncBackend for Platform {
        async fn begin_sync(&self, workspace_id: i64) -> Result<i64, BackendError>;
        async fn complete_sync(&self, id: i64, status: WorkspaceSyncStatus) -> Result<bool, BackendError>;
        async fn get_workspaces_sync_records(&self, workspace_id: i64) -> Result<Vec<WorkspaceSync>, BackendError>;
    }

    #[async_trait]
    impl WorkspaceAliasBackend for Platform {
        async fn add_alias(
            &self,
            workspace_id: i64,
            alias: &str,
        ) -> Result<i64, BackendError>;
        async fn get_aliases(
            &self,
            workspace_id: i64,
        ) -> Result<Vec<WorkspaceAlias>, BackendError>;
    }

}

#[async_trait]
impl ExposureBackend for MockPlatform {
    async fn insert(
        &self,
        workspace_id: i64,
        workspace_tag_id: Option<i64>,
        commit_id: &str,
        default_file_id: Option<i64>,
    ) -> Result<i64, BackendError> {
        self.exposure_insert(workspace_id, workspace_tag_id, commit_id, default_file_id).await
    }
    async fn list_for_workspace(
        &self,
        workspace_id: i64,
    ) -> Result<Exposures, BackendError> {
        self.exposure_list_for_workspace(workspace_id).await
    }
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<Exposure, BackendError> {
        self.exposure_get_id(id).await
    }
    async fn set_default_file(
        &self,
        id: i64,
        file_id: i64,
    ) -> Result<bool, BackendError> {
        self.set_default_file(id, file_id).await
    }
}

#[async_trait]
impl ExposureFileBackend for MockPlatform {
    async fn insert(
        &self,
        exposure_id: i64,
        workspace_file_path: &str,
        default_view_id: Option<i64>,
    ) -> Result<i64, BackendError> {
        self.exposure_file_insert(exposure_id, workspace_file_path, default_view_id).await
    }
    async fn list_for_exposure(
        &self,
        exposure_id: i64,
    ) -> Result<ExposureFiles, BackendError> {
        self.exposure_file_list_for_exposure(exposure_id).await
    }
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<ExposureFile, BackendError> {
        self.exposure_file_get_id(id).await
    }
    async fn set_default_view(
        &self,
        id: i64,
        file_id: i64,
    ) -> Result<bool, BackendError> {
        self.exposure_file_set_default_view(id, file_id).await
    }
}

#[async_trait]
impl ExposureFileViewBackend for MockPlatform {
    async fn insert(
        &self,
        exposure_file_id: i64,
        view_task_template_id: i64,
    ) -> Result<i64, BackendError> {
        self.exposure_file_view_insert(exposure_file_id, view_task_template_id).await
    }
    async fn list_for_exposure_file(
        &self,
        exposure_file_id: i64,
    ) -> Result<ExposureFileViews, BackendError> {
        self.exposure_file_view_list_for_exposure_file(exposure_file_id).await
    }
    async fn get_id(
        &self,
        id: i64,
    ) -> Result<ExposureFileView, BackendError> {
        self.exposure_file_view_get_id(id).await
    }
    async fn update_view_key(
        &self,
        id: i64,
        view_key: &str,
    ) -> Result<bool, BackendError> {
        self.exposure_file_view_update_view_key(id, view_key).await
    }
}
