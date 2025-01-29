use axum_login::AuthSession;
use pmrac::{
    error::Error as ACError,
    axum_login::{
        Authorization,
        Credentials,
    },
    Platform,
};
use pmrcore::ac::agent::Agent;
use crate::{
    enforcement::PolicyState,
    error::AppError,
};

pub struct Session(AuthSession<Platform>);

impl From<AuthSession<Platform>> for Session {
    fn from(value: AuthSession<Platform>) -> Self {
        Self(value)
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
}
