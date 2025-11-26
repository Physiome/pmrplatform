use pmrcore::workspace::Workspace;
use utoipa::OpenApi;

use crate::{
    enforcement::EnforcedOk,
    workspace::api::{
        __path_list_workspaces,
        Workspaces,
    },
};

#[derive(OpenApi)]
#[openapi(
    info(description = "OpenAPI description for pmrplatform"),
    paths(
        list_workspaces,
    ),
    components(schemas(
        EnforcedOk<Workspaces>,
        Workspace,
        Workspaces,
    )),
)]
pub struct ApiDoc;
