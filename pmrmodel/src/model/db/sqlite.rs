mod ac;
mod alias;

mod exposure;
mod exposure_file;
mod exposure_file_profile;
mod exposure_file_view;
mod exposure_file_view_task;
mod exposure_file_view_task_template;

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
