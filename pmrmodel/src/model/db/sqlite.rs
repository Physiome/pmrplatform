mod ac;
mod alias;

mod exposure;
mod exposure_file;
mod exposure_file_profile;
mod exposure_file_view;
mod exposure_file_view_task;
mod exposure_file_view_task_template;

mod idgen;

mod profile;

mod workspace;
mod workspace_sync;
mod workspace_tag;

mod task;
mod task_template;

mod default_impl {
    use pmrcore::platform::{
        DefaultACPlatform,
        DefaultMCPlatform,
        DefaultTMPlatform,
    };
    use crate::backend::db::SqliteBackend;

    impl DefaultACPlatform for SqliteBackend {}
    impl DefaultMCPlatform for SqliteBackend {}
    impl DefaultTMPlatform for SqliteBackend {}
}

// TODO eventually if we need to override certain implementation to better
// optimize selection:
//
// mod specialized {
//     use async_trait::async_trait;
//     use pmrcore::{
//         alias::AliasEntries,
//         error::BackendError,
//         platform::{
//             ACPlatform,
//             MCPlatform,
//             TMPlatform,
//         },
//         workspace::WorkspaceRef,
//     };
//
//     #[async_trait]
//     impl MCPlatform for SqliteBackend {
//         fn as_dyn(&self) -> &dyn MCPlatform {
//             self
//         }
//         async fn list_aliased_workspaces<'a>(
//             &'a self,
//         ) -> Result<AliasEntries<WorkspaceRef<'a>>, BackendError> {
//             todo!()
//         }
//     }
// }

#[cfg(test)]
pub(crate) mod testing {
    use pmrcore::{
        platform::MCPlatform,
        workspace::Workspace,
    };
    use crate::backend::db::{
        MigrationProfile,
        SqliteBackend,
    };

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrapp)
            .await?;
        let entry = backend.create_aliased_workspace(
            "https://models.example.com".into(),
            "".into(),
            "".into(),
        ).await?;
        assert_eq!(entry.alias, "1");
        let answer = Workspace {
            id: 1,
            url: "https://models.example.com".into(),
            superceded_by_id: None,
            created_ts: 1234567890,
            description: Some("".into()),
            long_description: Some("".into()),
            exposures: None,
        };
        assert_eq!(entry.entity.into_inner(), answer);
        Ok(())
    }
}
