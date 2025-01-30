use axum_login::{
    AuthSession,
    Error as AxumLoginError,
};
use pmrac::{
    error::Error as ACError,
    axum_login::{
        Authorization,
        Credentials,
    },
    Platform,
};
use pmrcore::ac::{
    agent::Agent,
    user::User,
};
use crate::{
    enforcement::PolicyState,
    error::{
        AppError,
        AuthError,
    },
};

pub struct Session(AuthSession<Platform>);

impl From<AuthSession<Platform>> for Session {
    fn from(value: AuthSession<Platform>) -> Self {
        Self(value)
    }
}

impl From<Session> for AuthSession<Platform> {
    fn from(value: Session) -> Self {
        value.0
    }
}

impl Session {
    pub async fn enforcer(
        &self,
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> Result<(), AppError> {
        let backend = &self.0.backend;
        let agent: Agent = self.0.user
            .as_ref()
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
        &self,
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> Result<PolicyState, AppError> {
        let backend = &self.0.backend;
        let agent: Agent = self.0.user
            .as_ref()
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

    pub async fn sign_in_with_login_password(
        &mut self,
        login: String,
        password: String,
    ) -> Result<String, AuthError> {
        let creds = Credentials {
            authorization: Authorization::LoginPassword(login, password),
            origin: "localhost".to_string(),  // TODO plug in remote host
        };
        match self.0.authenticate(creds).await {
            Ok(Some(auth)) => {
                self.0.login(&auth).await
                    .map_err(|_| AuthError::InternalServerError)?;
                Ok("You are logged in.".to_string())
            },
            Ok(None) | Err(AxumLoginError::Backend(ACError::Authentication(_))) => {
                // TODO handle restricted account error differently?
                Err(AuthError::InvalidCredentials)
            },
            Err(_) => Err(AuthError::InternalServerError),
        }
    }

    pub async fn sign_out(&mut self) -> Result<(), AuthError> {
        // TODO figure out if we want to automatically purge historical records
        // upon logout
        /*
        match self.0.logout().await {
            Ok(Some(s)) => s.logout().await
                .map_err(|_| AuthError::InternalServerError),
            _ => Err(AuthError::InternalServerError),
        }
        */

        self.0.logout().await
            .map(|_| ())
            .map_err(|_| AuthError::InternalServerError)
    }

    pub fn current_user(&self) -> Option<User> {
        self.0.user
            .as_ref()
            .map(|auth| auth.user().clone_inner())
    }
}

pub async fn session() -> Result<Session, AppError> {
    Ok(leptos_axum::extract::<axum::Extension<AuthSession<Platform>>>()
        .await
        .map_err(|_| AppError::InternalServerError)?
        .0
        .into()
    )
}
