use leptos::{
    prelude::ServerFnError,
    server,
};
use pmrcore::ac::user::User;
use std::{
    convert::Infallible,
    fmt,
    str::FromStr,
};

#[cfg(feature = "ssr")]
mod ssr {
    pub use axum_login::{
        AuthSession,
        Error as AxumLoginError,
    };
    pub use pmrac::{
        error::Error as ACError,
        axum_login::{
            Authorization,
            Credentials,
        },
        Platform,
    };
    use pmrcore::ac::agent::Agent;
    pub use pmrcore::ac::workflow::State;
    pub use crate::{
        server::platform,
        workflow::state::TRANSITIONS,
    };
    use crate::{
        enforcement::PolicyState,
        error::AppError,
    };

    pub async fn session() -> Result<AuthSession<Platform>, AppError> {
        Ok(leptos_axum::extract::<axum::Extension<AuthSession<Platform>>>()
            .await
            .map_err(|_| AppError::InternalServerError)?
            .0
        )
    }

    pub async fn enforcer(
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> Result<(), AppError> {
        let session = session().await?;
        let backend = session.backend;
        let agent: Agent = session.user
            .map(|auth| auth.user().into())
            .unwrap_or(Agent::Anonymous);
        let resource = resource.into();
        let action = action.into();
        log::trace!("enforce on: agent={agent} resource={resource:?} action={action:?}");
        if backend
            .enforce(agent.clone(), &resource, action)
            .await
            .map_err(|_| AppError::InternalServerError)?
        {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }

    pub async fn enforcer_and_policy_state(
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> Result<PolicyState, AppError> {
        let session = session().await?;
        let backend = session.backend;
        let agent: Agent = session.user
            .map(|auth| auth.user().into())
            .unwrap_or(Agent::Anonymous);
        let resource = resource.into();
        let action = action.into();
        log::trace!("enforce on: agent={agent} resource={resource:?} action={action:?}");
        let (policy, result) = backend
            .get_policy_and_enforce(agent.clone(), &resource, action)
            .await
            .map_err(|_| AppError::InternalServerError)?;
        if result {
            let state = backend
                .get_wf_state_for_res(&resource)
                .await
                .map_err(|_| AppError::InternalServerError)?;
            Ok(PolicyState::new(Some(policy), state))
        } else {
            Err(AppError::Forbidden)
        }
    }
}

use crate::{
    enforcement::PolicyState,
    error::AppError,
};

#[cfg(feature = "ssr")]
pub use self::ssr::*;

#[derive(Debug, Copy, Clone)]
pub enum AuthError {
    InternalServerError,
    InvalidCredentials,
}

impl From<AuthError> for &'static str {
    fn from(v: AuthError) -> &'static str {
        match v {
            AuthError::InternalServerError => "Internal server error",
            AuthError::InvalidCredentials => "Invalid credentials provided",
        }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", <&'static str>::from(*self))
    }
}

impl FromStr for AuthError {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Invalid credentials provided" => AuthError::InvalidCredentials,
            _ => AuthError::InternalServerError,
        })
    }
}

impl From<ServerFnError<AuthError>> for AuthError {
    fn from(e: ServerFnError<AuthError>) -> Self {
        match e {
            ServerFnError::WrappedServerError(e) => e,
            _ => Self::InternalServerError,
        }
    }
}

#[server]
pub(crate) async fn sign_in_with_login_password(
    // FIXME figure out how to best approach CSRF; maybe this be best moved to the
    // middleware.
    login: String,
    password: String,
) -> Result<String, ServerFnError<AuthError>> {
    let mut session = session().await
        .map_err(|_| AuthError::InternalServerError)?;
    let creds = Credentials {
        authorization: Authorization::LoginPassword(login, password),
        origin: "localhost".to_string(),  // TODO plug in remote host
    };
    match session.authenticate(creds).await {
        Ok(Some(auth)) => {
            session.login(&auth).await
                .map_err(|_| AuthError::InternalServerError)?;
            Ok("You are logged in.".to_string())
        },
        Ok(None) | Err(AxumLoginError::Backend(ACError::Authentication(_))) => {
            // TODO handle restricted account error differently?
            Err(AuthError::InvalidCredentials.into())
        },
        Err(_) => Err(AuthError::InternalServerError.into()),
    }
}

#[server]
pub(crate) async fn sign_out() -> Result<(), ServerFnError> {
    let mut session = session().await?;
    session.logout().await?;
    leptos_axum::redirect("/logged_out");
    Ok(())
}

#[server]
pub(crate) async fn current_user() -> Result<Option<User>, ServerFnError> {
    let session = session().await?;
    Ok(session.user.map(|auth| auth.user().clone_inner()))
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
