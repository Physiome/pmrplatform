mod data {
    use async_trait::async_trait;
    use crate::error::Error;

    #[async_trait]
    pub trait Workspace<'a, S> {
        fn id(&self) -> i64;
        fn url(&self) -> &str;
        fn superceded_by_id(&self) -> Option<i64>;
        fn description(&self) -> Option<&str>;
        fn long_description(&self) -> Option<&str>;
        fn created_ts(&self) -> i64;
        async fn exposures(&'a self) -> Result<&'a S, Error>;
    }
}

mod backend {
    use async_trait::async_trait;
    use crate::{
        error::BackendError,
        workspace::{
            Workspace,
            Workspaces,
            WorkspaceSync,
            WorkspaceSyncStatus,
            WorkspaceTag,
        },
    };

    #[async_trait]
    pub trait WorkspaceBackend {
        async fn add_workspace(
            &self,
            url: &str,
            description: &str,
            long_description: &str,
        ) -> Result<i64, BackendError>;
        async fn update_workspace(
            &self,
            id: i64,
            description: &str,
            long_description: &str,
        ) -> Result<bool, BackendError>;
        async fn get_workspace_by_id(
            &self,
            id: i64,
        ) -> Result<Workspace, BackendError>;
        async fn list_workspaces(
            &self,
        ) -> Result<Workspaces, BackendError>;
        async fn list_workspace_by_url(
            &self,
            url: &str,
        ) -> Result<Workspaces, BackendError>;
    }

    #[async_trait]
    pub trait WorkspaceSyncBackend {
        async fn begin_sync(
            &self,
            workspace_id: i64,
        ) -> Result<i64, BackendError>;
        async fn complete_sync(
            &self,
            id: i64,
            status: WorkspaceSyncStatus,
        ) -> Result<bool, BackendError>;
        async fn get_workspaces_sync_records(
            &self,
            workspace_id: i64,
        ) -> Result<Vec<WorkspaceSync>, BackendError>;
    }

    #[async_trait]
    pub trait WorkspaceTagBackend {
        async fn index_workspace_tag(
            &self,
            workspace_id: i64,
            name: &str,
            commit_id: &str,
        ) -> Result<i64, BackendError>;
        async fn get_workspace_tags(
            &self,
            workspace_id: i64,
        ) -> Result<Vec<WorkspaceTag>, BackendError>;
    }
}

pub use data::Workspace;
pub use backend::{
    WorkspaceBackend,
    WorkspaceSyncBackend,
    WorkspaceTagBackend,
};
