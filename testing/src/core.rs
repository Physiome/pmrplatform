use async_trait::async_trait;
use mockall::mock;
use pmrcore::{
    error::{
        BackendError,
        Error,
        task::TaskError,
    },
    exposure::{
        Exposure,
        Exposures,
        ExposureFile,
        ExposureFiles,
        ExposureFileView,
        ExposureFileViews,
        profile::{
            ExposureFileProfile,
            traits::ExposureFileProfileBackend,
        },
        traits::{
            ExposureBackend,
            ExposureFileBackend,
            ExposureFileViewBackend,
        },
        task::{
            ExposureFileViewTask,
            traits::{
                ExposureTaskBackend,
                ExposureTaskTemplateBackend,
            },
        },
    },
    platform::PlatformUrl,
    task::{
        Task,
        traits::TaskBackend,
    },
    task_template::{
        TaskTemplate,
        TaskTemplateArg,
        TaskTemplateArgChoice,
        UserInputMap,
        traits::TaskTemplateBackend,
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
    profile::{
        ViewTaskTemplateProfile,
        ViewTaskTemplates,
        ViewTaskTemplate,
        Profile,
        traits::{
            ProfileBackend,
            ViewTaskTemplateBackend,
            ProfileViewsBackend,
            ViewTaskTemplateProfileBackend,
        },
    },
};

mock! {
    pub Platform {
        pub async fn exposure_insert<'a>(
            &self,
            description: Option<&'a str>,
            workspace_id: i64,
            workspace_tag_id: Option<i64>,
            commit_id: &str,
            default_file_id: Option<i64>,
        ) -> Result<i64, BackendError>;
        pub async fn exposure_list(
            &self,
        ) -> Result<Exposures, BackendError>;
        pub async fn exposure_list_for_workspace(
            &self,
            workspace_id: i64,
        ) -> Result<Exposures, BackendError>;
        pub async fn exposure_get_id(
            &self,
            id: i64,
        ) -> Result<Exposure, BackendError>;
        pub async fn exposure_set_default_file(
            &self,
            id: i64,
            file_id: i64,
        ) -> Result<bool, BackendError>;

        pub async fn exposure_file_insert(
            &self,
            exposure_id: i64,
            workspace_file_path: &str,
            default_view_id: Option<i64>,
        ) -> Result<i64, BackendError>;
        pub async fn exposure_file_list_for_exposure(
            &self,
            exposure_id: i64,
        ) -> Result<ExposureFiles, BackendError>;
        pub async fn exposure_file_get_id(
            &self,
            id: i64,
        ) -> Result<ExposureFile, BackendError>;
        pub async fn exposure_file_get_by_exposure_filepath(
            &self,
            exposure_id: i64,
            workspace_file_path: &str,
        ) -> Result<ExposureFile, BackendError>;
        pub async fn exposure_file_set_default_view(
            &self,
            id: i64,
            file_id: i64,
        ) -> Result<bool, BackendError>;

        pub async fn exposure_file_view_insert(
            &self,
            exposure_file_id: i64,
            view_task_template_id: i64,
            exposure_file_view_task_id: Option<i64>,
        ) -> Result<i64, BackendError>;
        pub async fn exposure_file_view_list_for_exposure_file(
            &self,
            exposure_file_id: i64,
        ) -> Result<ExposureFileViews, BackendError>;
        pub async fn exposure_file_view_get_id(
            &self,
            id: i64,
        ) -> Result<ExposureFileView, BackendError>;
        pub async fn exposure_file_view_get_by_file_view_template(
            &self,
            exposure_file_id: i64,
            view_task_template_id: i64,
        ) -> Result<ExposureFileView, BackendError>;
        pub async fn exposure_file_view_get_by_file_view_key(
            &self,
            exposure_file_id: i64,
            view_key: &str,
        ) -> Result<ExposureFileView, BackendError>;
        pub async fn exposure_file_view_update_view_key<'a>(
            &'a self,
            id: i64,
            view_key: Option<&'a str>,
        ) -> Result<bool, BackendError>;
        pub async fn exposure_file_view_update_exposure_file_view_task_id(
            &self,
            id: i64,
            exposure_file_view_task_id: Option<i64>,
        ) -> Result<bool, BackendError>;
        pub async fn exposure_file_view_select_id_by_task_id(
            &self,
            task_id: i64,
        ) -> Result<i64, BackendError>;
        pub async fn exposure_task_set_file_templates(
            &self,
            exposure_file_id: i64,
            task_template_ids: &[i64],
        ) -> Result<(), BackendError>;
        pub async fn exposure_task_get_file_templates(
            &self,
            exposure_file_id: i64,
        ) -> Result<Vec<ViewTaskTemplate>, BackendError>;
    }

    #[async_trait]
    impl ExposureFileProfileBackend for Platform {
        async fn set_ef_profile(
            &self,
            exposure_file_id: i64,
            profile_id: i64,
        ) -> Result<(), BackendError>;
        async fn get_ef_profile(
            &self,
            exposure_file_id: i64,
        ) -> Result<ExposureFileProfile, BackendError>;
        async fn update_ef_user_input(
            &self,
            exposure_file_id: i64,
            user_input: &UserInputMap,
        ) -> Result<(), BackendError>;
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

    #[async_trait]
    impl ProfileBackend for Platform {
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
        async fn list_profiles(
            &self,
        ) -> Result<Vec<Profile>, BackendError>;
        // TODO listing/query for set of profiles.
        // This may be implemented at the backends for the linked types.
    }

    #[async_trait]
    impl ViewTaskTemplateBackend for Platform {
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
    impl ProfileViewsBackend for Platform {
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
    impl ViewTaskTemplateProfileBackend for Platform {
        // TODO determine if a default implementation for the combination
        // of the previous two traits be feasible.
        async fn get_view_task_template_profile(
            &self,
            profile_id: i64,
        ) -> Result<ViewTaskTemplateProfile, BackendError>;
    }

    #[async_trait]
    impl ExposureTaskBackend for Platform {
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

    #[async_trait]
    impl TaskBackend for Platform {
        async fn adds_task(
            &self,
            task: Task,
        ) -> Result<Task, TaskError>;
        async fn gets_task(
            &self,
            id: i64,
        ) -> Result<Task, BackendError>;
        async fn start(
            &self,
        ) -> Result<Option<Task>, BackendError>;
        async fn run(
            &self,
            id: i64,
            pid: i64,
        ) -> Result<bool, BackendError>;
        async fn complete(
            &self,
            id: i64,
            exit_status: i64,
        ) -> Result<bool, BackendError>;
    }

    impl PlatformUrl for Platform {
        fn url(&self) -> &str;
    }
}

#[async_trait]
impl ExposureBackend for MockPlatform {
    async fn insert(
        &self,
        description: Option<&str>,
        workspace_id: i64,
        workspace_tag_id: Option<i64>,
        commit_id: &str,
        default_file_id: Option<i64>,
    ) -> Result<i64, BackendError> {
        self.exposure_insert(description, workspace_id, workspace_tag_id, commit_id, default_file_id).await
    }
    async fn list(
        &self,
    ) -> Result<Exposures, BackendError> {
        self.exposure_list().await
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
    async fn get_by_exposure_filepath(
        &self,
        exposure_id: i64,
        workspace_file_path: &str,
    ) -> Result<ExposureFile, BackendError> {
        self.exposure_file_get_by_exposure_filepath(exposure_id, workspace_file_path).await
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
        exposure_file_view_task_id: Option<i64>,
    ) -> Result<i64, BackendError> {
        self.exposure_file_view_insert(
            exposure_file_id,
            view_task_template_id,
            exposure_file_view_task_id,
        ).await
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
    async fn get_by_file_view_template(
        &self,
        exposure_file_id: i64,
        view_task_template_id: i64,
    ) -> Result<ExposureFileView, BackendError> {
        self.exposure_file_view_get_by_file_view_template(
            exposure_file_id,
            view_task_template_id,
        ).await
    }
    async fn get_by_file_view_key(
        &self,
        exposure_file_id: i64,
        view_key: &str,
    ) -> Result<ExposureFileView, BackendError> {
        self.exposure_file_view_get_by_file_view_key(
            exposure_file_id,
            view_key,
        ).await
    }
    async fn update_view_key(
        &self,
        id: i64,
        view_key: Option<&str>,
    ) -> Result<bool, BackendError> {
        self.exposure_file_view_update_view_key(id, view_key).await
    }
    async fn update_exposure_file_view_task_id(
        &self,
        id: i64,
        exposure_file_view_task_id: Option<i64>,
    ) -> Result<bool, BackendError> {
        self.exposure_file_view_update_exposure_file_view_task_id(
            id,
            exposure_file_view_task_id,
        ).await
    }
    async fn select_id_by_task_id(
        &self,
        task_id: i64,
    ) -> Result<i64, BackendError> {
        self.exposure_file_view_select_id_by_task_id(
            task_id
        ).await
    }
}

#[async_trait]
impl ExposureTaskTemplateBackend for MockPlatform {
    async fn set_file_templates(
        &self,
        exposure_file_id: i64,
        task_template_ids: &[i64],
    ) -> Result<(), BackendError> {
        self.exposure_task_set_file_templates(
            exposure_file_id,
            task_template_ids,
        ).await
    }
    async fn get_file_templates(
        &self,
        exposure_file_id: i64,
    ) -> Result<Vec<ViewTaskTemplate>, BackendError> {
        self.exposure_task_get_file_templates(exposure_file_id).await
    }
}

#[async_trait]
impl TaskTemplateBackend for MockPlatform {
    async fn add_task_template(
        &self,
        _bin_path: &str,
        _version_id: &str,
    ) -> Result<(i64, i64), BackendError> {
        unimplemented!()
    }
    async fn finalize_new_task_template(
        &self,
        _id: i64,
    ) -> Result<Option<i64>, BackendError> {
        unimplemented!()
    }
    async fn add_task_template_arg(
        &self,
        _task_template_id: i64,
        _flag: Option<&str>,
        _flag_joined: bool,
        _flag_omit_when_null: bool,
        _prompt: Option<&str>,
        _default: Option<&str>,
        _choice_fixed: bool,
        _choice_source: Option<&str>,
    ) -> Result<i64, BackendError> {
        unimplemented!()
    }
    async fn delete_task_template_arg_by_id(
        &self,
        _id: i64,
    ) -> Result<Option<TaskTemplateArg>, BackendError> {
        unimplemented!()
    }
    async fn add_task_template_arg_choice(
        &self,
        _task_template_arg_id: i64,
        _to_arg: Option<&str>,
        _label: &str,
    ) -> Result<i64, BackendError> {
        unimplemented!()
    }
    async fn get_task_template_arg_by_id(
        &self,
        _id: i64,
    ) -> Result<Option<TaskTemplateArg>, BackendError> {
        unimplemented!()
    }
    async fn delete_task_template_arg_choice_by_id(
        &self,
        _id: i64,
    ) -> Result<Option<TaskTemplateArgChoice>, BackendError> {
        unimplemented!()
    }
    async fn get_task_template_by_id(
        &self,
        _id: i64,
    ) -> Result<TaskTemplate, BackendError> {
        unimplemented!()
    }
    async fn get_task_template_by_arg_id(
        &self,
        _id: i64,
    ) -> Result<TaskTemplate, BackendError> {
        unimplemented!()
    }
}
