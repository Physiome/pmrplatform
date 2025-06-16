use leptos::{
    prelude::ServerFnError,
    server,
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

#[server]
pub(crate) async fn sign_in_with_login_password(
    login: String,
    password: String,
) -> Result<String, AuthError> {
    let mut session = session().await
        .map_err(|_| AuthError::InternalServerError)?;
    // FIXME figure out how to best approach CSRF; maybe this be best moved to the
    // middleware.
    Ok(session.sign_in_with_login_password(login, password).await?)
}

#[server]
pub(crate) async fn sign_out() -> Result<(), AuthError> {
    let mut session = session().await
        .map_err(|_| AuthError::InternalServerError)?;
    session.sign_out().await?;
    leptos_axum::redirect("/auth/logged_out");
    Ok(())
}

#[server]
pub(crate) async fn current_user() -> Result<Option<User>, ServerFnError> {
    Ok(session().await?
        .current_user())
}

#[server]
pub(crate) async fn workflow_transition(
    resource: String,
    target: String,
) -> Result<PolicyState, ServerFnError<AppError>> {
    if let Some(user) = current_user().await
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
