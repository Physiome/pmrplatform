use pmrcore::{
    profile::UserPromptGroup,
    task_template::UserInputMap,
    workspace::Workspace,
};
use utoipa::{
    openapi::{
        self,
        security::{
            ApiKey,
            ApiKeyValue,
            SecurityScheme,
        },
    },
    Modify,
    OpenApi,
};

use crate::{
    ac::api::{
        __path_current_user,
        __path_sign_in_with_login_password,
        __path_sign_out,
        __path_workflow_transition,
    },
    app::id::Id,
    enforcement::EnforcedOk,
    exposure::api::{
        Exposures,
        ExposureInfo,
        WizardInfo,
        __path_list_exposures,
        __path_list_aliased_exposures,
        __path_list_aliased_exposures_for_workspace,
        __path_get_exposure_info,
        __path_create_exposure_openapi,
        __path_wizard,
        __path_wizard_add_file_openapi,
        __path_wizard_build_openapi,
    },
    server::exposure::{
        __path_exposure_file_data,
        __path_exposure_file_safe_html,
        __path_wizard_field_update,
    },
    workspace::api::{
        __path_create_workspace_core,
        __path_list_workspaces,
        __path_list_aliased_workspaces,
        __path_get_log_info,
        __path_get_workspace_info,
        __path_workspace_root_policy_state,
        __path_synchronize_openapi,
        Workspaces,
    },
};

struct Security;

impl Modify for Security {
    fn modify(&self, openapi: &mut openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "cookie",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::with_description(
                    "id",
                    "This is typically set as a `HttpOnly` cookie via the `sign_in_with_login_password` \
                    endpoint.",
                ))),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&Security),
    info(description = "OpenAPI description for pmrplatform"),
    paths(
        // Auth
        current_user,
        sign_in_with_login_password,
        sign_out,
        workflow_transition,

        // Exposures
        exposure_file_data,
        exposure_file_safe_html,
        list_exposures,
        list_aliased_exposures,
        list_aliased_exposures_for_workspace,
        get_exposure_info,
        create_exposure_openapi,
        wizard,
        wizard_add_file_openapi,
        wizard_build_openapi,
        wizard_field_update,

        // Workspaces
        create_workspace_core,
        list_workspaces,
        list_aliased_workspaces,
        get_log_info,
        get_workspace_info,
        workspace_root_policy_state,
        synchronize_openapi,
    ),
    components(schemas(
        EnforcedOk<Workspaces>,
        Exposures,
        ExposureInfo,
        Id,
        UserInputMap,
        UserPromptGroup,
        WizardInfo,
        Workspace,
        Workspaces,
    )),
)]
pub struct ApiDoc;
