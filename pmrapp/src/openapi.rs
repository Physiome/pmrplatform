use pmrcore::workspace::Workspace;
use utoipa::OpenApi;

use crate::{
    app::id::Id,
    enforcement::EnforcedOk,
    workspace::api::{
        __path_create_workspace,
        __path_list_workspaces,
        __path_list_aliased_workspaces,
        __path_get_log_info,
        __path_get_workspace_info,
        __path_workspace_root_policy_state,
        Workspaces,
    },
};

#[derive(OpenApi)]
#[openapi(
    info(description = "OpenAPI description for pmrplatform"),
    paths(
        create_workspace,
        list_workspaces,
        list_aliased_workspaces,
        get_log_info,
        get_workspace_info,
        workspace_root_policy_state,
    ),
    components(schemas(
        EnforcedOk<Workspaces>,
        Id,
        Workspace,
        Workspaces,
    )),
)]
pub struct ApiDoc;
