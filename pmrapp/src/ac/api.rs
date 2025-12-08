use leptos::{
    prelude::ServerFnError,
    server,
    server_fn,
};
use pmrcore::ac::user::User;

#[cfg(feature = "ssr")]
mod ssr {
    pub use pmrcore::ac::workflow::State;
    pub use std::str::FromStr;
    pub use crate::{
        server::platform,
        server::ac::session,
        workflow::state::TRANSITIONS,
    };
}

use crate::{
    enforcement::PolicyState,
    error::{
        AppError,
        AuthError,
    },
};

#[cfg(feature = "ssr")]
pub use self::ssr::*;

// this struct is a placeholder to help utoipa
#[cfg(feature = "utoipa")]
#[allow(dead_code)]
#[derive(utoipa::ToSchema)]
struct LoginPassword {
    login: String,
    password: String,
}

/// Acquire a bearer token from login/password
#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/bearer/from_login_password",
    request_body(
        description = r#"
Acquire a bearer token from login/password.
        "#,
        content((
            LoginPassword = "application/json",
        )),
    ),
    responses((
        status = 200,
        description = "The bearer token.",
        body = String,
        example = "abcdefghijkKJIHGFEDCBA",
    ), AppError),
))]
#[server(
    endpoint = "bearer/from_login_password",
    input = server_fn::codec::Json,
)]
pub async fn bearer_from_login_password(
    login: String,
    password: String,
) -> Result<String, AuthError> {
    let mut session = session().await
        .map_err(|_| AuthError::InternalServerError)?;
    session.sign_in_with_login_password(login, password).await?
        .ok_or(AuthError::InternalServerError)
}

/// Sets a `SameSite=Strict` session cookie on success.
#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/sign_in_with_login_password",
    request_body(
        description = r#"
Sign in with login and password.
        "#,
        content((
            LoginPassword = "application/x-www-form-urlencoded",
        )),
    ),
    responses((
        status = 200,
        description = "Message describing the outcome.",
        body = String,
        example = "You are logged in.",
    ), AppError),
))]
#[server(
    endpoint = "sign_in_with_login_password",
)]
pub async fn sign_in_with_login_password(
    login: String,
    password: String,
) -> Result<String, AuthError> {
    let mut session = session().await
        .map_err(|_| AuthError::InternalServerError)?;
    // FIXME figure out how to best approach CSRF; maybe this be best moved to the
    // middleware.
    session.sign_in_with_login_password(login, password).await?;
    Ok("You are logged in.".to_string())
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/sign_out",
    responses((
        status = 200,
        description = "Status code means success.",
        body = (),
    ), AppError),
    security(
        (),
        ("cookie" = []),
        ("bearer" = []),
    ),
))]
#[server(
    endpoint = "sign_out",
)]
pub async fn sign_out() -> Result<(), AuthError> {
    let mut session = session().await
        .map_err(|_| AuthError::InternalServerError)?;
    session.sign_out().await?;
    leptos_axum::redirect("/auth/logged_out");
    Ok(())
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/current_user",
    responses((
        status = 200,
        description = "The current user.",
        body = Option<User>,
    ), AppError),
    security(
        (),
        ("cookie" = []),
        ("bearer" = []),
    ),
))]
#[server(
    endpoint = "current_user",
)]
pub async fn current_user() -> Result<Option<User>, ServerFnError> {
    Ok(session().await?
        .current_user())
}

// this struct is a placeholder to help utoipa
#[cfg(feature = "utoipa")]
#[allow(dead_code)]
#[derive(utoipa::ToSchema)]
struct WorkflowTransitionArgs {
    /// The resource to have the workflow state updated.
    resource: String,
    /// The target state.
    target: pmrcore::ac::workflow::State,
}

#[cfg_attr(feature = "utoipa", utoipa::path(
    post,
    path = "/api/workflow_transition",
    request_body(
        description = r#"
Update the workflow state for a given resource.
        "#,
        content((
            WorkflowTransitionArgs = "application/x-www-form-urlencoded",
        )),
    ),
    responses((
        status = 200,
        description = "The new `PolicyState` of the resource.",
        body = PolicyState,
    ), AppError),
    security(
        ("cookie" = []),
        ("bearer" = []),
    ),
))]
#[server(
    endpoint = "workflow_transition",
)]
pub async fn workflow_transition(
    resource: String,
    target: String,
) -> Result<PolicyState, AppError> {
    if let Some(user) = current_user().await
        // TODO figure out how to actually get 404 status code working here.
        .map_err(|_| AppError::Forbidden)?
    {
        let target_state = State::from_str(&target)
            .expect("State::from_str shouldn't have failed!");
        let platform = platform().await
            .map_err(|_| AppError::InternalServerError)?;
        let state = platform
            .ac_platform
            .get_wf_state_for_res(&resource)
            .await
            .map_err(|_| AppError::InternalServerError)?;
        let roles = platform
            .ac_platform
            .generate_policy_for_agent_res(&user.clone().into(), resource.clone())
            .await
            .map_err(|_| AppError::InternalServerError)?
            .to_roles();
        if TRANSITIONS.validate(roles, state, target_state) {
            platform.ac_platform.set_wf_state_for_res(&resource, target_state).await
                .map_err(|_| AppError::InternalServerError)?;
            let policy = platform
                .ac_platform
                .generate_policy_for_agent_res(&user.into(), resource)
                .await
                .map_err(|_| AppError::InternalServerError)?;
            Ok(PolicyState::new(Some(policy), target_state))
        } else {
            Err(AppError::Forbidden)?
        }
    } else {
        Err(AppError::Forbidden)?
    }
}
