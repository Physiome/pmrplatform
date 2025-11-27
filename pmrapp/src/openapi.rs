use pmrcore::workspace::Workspace;
use utoipa::OpenApi;

use crate::{
    app::id::Id,
    enforcement::EnforcedOk,
    workspace::api::{
        __path_list_workspaces,
        __path_list_aliased_workspaces,
        __path_get_workspace_info,
        __path_workspace_root_policy_state,
        Workspaces,
    },
};

#[derive(OpenApi)]
#[openapi(
    info(description = "OpenAPI description for pmrplatform"),
    paths(
        list_workspaces,
        list_aliased_workspaces,
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
