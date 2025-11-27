use pmrcore::workspace::Workspace;
use utoipa::OpenApi;

use crate::{
    enforcement::EnforcedOk,
    workspace::api::{
        __path_list_workspaces,
        __path_list_aliased_workspaces,
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
        workspace_root_policy_state,
    ),
    components(schemas(
        EnforcedOk<Workspaces>,
        Workspace,
        Workspaces,
    )),
)]
pub struct ApiDoc;
