use leptos::{
    prelude::ServerFnError,
    server,
};
use pmrcore::ac::user::User;

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
    use crate::error::AppError;
    use super::*;

    pub async fn session() -> Result<AuthSession<Platform>, ServerFnError> {
        Ok(leptos_axum::extract::<axum::Extension<AuthSession<Platform>>>()
            .await?
            .0
        )
    }

    pub async fn enforcer(
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> Result<(), ServerFnError> {
        let session = session().await?;
        let backend = session.backend;
        let agent: Agent = session.user
            .map(|auth| auth.user().into())
            .unwrap_or(Agent::Anonymous);
        let resource = resource.into();
        let action = action.into();
        log::trace!("enforce on: agent={agent} resource={resource:?} action={action:?}");
        if backend.enforce(
            agent,
            resource,
            action,
        ).await? {
            Ok(())
        } else {
            Err(AppError::Forbidden)?
        }
    }
}
#[cfg(feature = "ssr")]
pub use self::ssr::*;

#[server]
pub(crate) async fn sign_in_with_login_password(
    // FIXME figure out how to best approach CSRF; maybe this be best moved to the
    // middleware.
    login: String,
    password: String,
) -> Result<bool, ServerFnError> {
    let mut session = session().await?;
    let creds = Credentials {
        authorization: Authorization::LoginPassword(login, password),
        origin: "localhost".to_string(),  // TODO plug in remote host
    };
    match session.authenticate(creds).await {
        Ok(Some(auth)) => {
            session.login(&auth).await?;
            Ok(true)
        },
        Ok(None) | Err(AxumLoginError::Backend(ACError::Authentication(_))) => {
            // TODO handle restricted account error differently?
            Ok(false)
        },
        Err(e) => Err(e)?,
    }
}

#[server]
pub(crate) async fn sign_out() -> Result<(), ServerFnError> {
    let mut session = session().await?;
    session.logout().await?;
    Ok(())
}

#[server]
pub(crate) async fn current_user() -> Result<Option<User>, ServerFnError> {
    let session = session().await?;
    Ok(session.user.map(|auth| auth.user().clone_inner()))
}
