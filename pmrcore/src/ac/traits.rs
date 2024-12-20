use async_trait::async_trait;
use crate::error::BackendError;
use super::{
    agent::Agent,
    genpolicy::Policy,
    role::Role,
    session::{
        Session,
        SessionToken,
    },
    user::User,
    workflow::State,
};

#[async_trait]
pub trait UserBackend {
    async fn add_user(
        &self,
        name: &str,
    ) -> Result<i64, BackendError>;
    async fn get_user_by_id(
        &self,
        id: i64,
    ) -> Result<Option<User>, BackendError>;
    async fn get_user_by_name(
        &self,
        name: &str,
    ) -> Result<Option<User>, BackendError>;
    async fn get_user_password(
        &self,
        user_id: i64,
    ) -> Result<String, BackendError>;
    async fn store_user_password(
        &self,
        user_id: i64,
        password: &str,
    ) -> Result<i64, BackendError>;
    async fn purge_user_passwords(
        &self,
        user_id: i64,
    ) -> Result<(), BackendError>;
}

#[async_trait]
pub trait PolicyBackend {
    async fn grant_role_to_user(
        &self,
        user: &User,
        role: Role,
    ) -> Result<bool, BackendError>;
    async fn revoke_role_from_user(
        &self,
        user: &User,
        role: Role,
    ) -> Result<bool, BackendError>;
    async fn get_roles_for_user(
        &self,
        user: &User,
    ) -> Result<Vec<Role>, BackendError>;

    async fn res_grant_role_to_agent(
        &self,
        res: &str,
        agent: &Agent,
        role: Role,
    ) -> Result<bool, BackendError>;
    async fn res_revoke_role_from_agent(
        &self,
        res: &str,
        agent: &Agent,
        role: Role,
    ) -> Result<bool, BackendError>;
    async fn get_res_grants_for_res(
        &self,
        res: &str,
    ) -> Result<Vec<(Agent, Vec<Role>)>, BackendError>;
    async fn get_res_grants_for_agent(
        &self,
        agent: &Agent,
    ) -> Result<Vec<(String, Vec<Role>)>, BackendError>;

    async fn assign_policy_to_wf_state(
        &self,
        wf_state: State,
        role: Role,
        action: &str,
    ) -> Result<(), BackendError>;
    async fn remove_policy_from_wf_state(
        &self,
        wf_state: State,
        role: Role,
        action: &str,
    ) -> Result<(), BackendError>;
}

#[async_trait]
pub trait ResourceBackend {
    async fn get_wf_state_for_res(
        &self,
        res: &str,
    ) -> Result<State, BackendError>;
    async fn set_wf_state_for_res(
        &self,
        res: &str,
        wf_state: State,
    ) -> Result<(), BackendError>;
    async fn generate_policy_for_agent_res(
        &self,
        agent: &Agent,
        res: String,
    ) -> Result<Policy, BackendError>;
}

#[async_trait]
pub trait SessionBackend {
    async fn save_session(
        &self,
        session: &Session,
    ) -> Result<i64, BackendError>;
    async fn load_session(
        &self,
        token: SessionToken,
    ) -> Result<Session, BackendError>;
    async fn purge_session(
        &self,
        token: SessionToken,
    ) -> Result<(), BackendError>;
    /// Get sessions for the user
    ///
    /// Note that this returns just the session, tokens will be of the
    /// empty type.
    async fn get_user_sessions(
        &self,
        user_id: i64,
    ) -> Result<Vec<Session>, BackendError>;
    /// Purge all sessions for the user
    ///
    /// Optionally a token may be provided to omit from the purge, e.g.
    /// to keep a current session alive.
    async fn purge_user_sessions(
        &self,
        user_id: i64,
        keep_token: Option<SessionToken>,
    ) -> Result<(), BackendError>;
}

/// A trait for typical enforcers
///
/// This is the generic enforcer trait, where the enforcement is done on
/// the provided agent, resource and endpoint_group, and the underlying
/// type is assumed to be able to fully provide a pass or fail result.
pub trait Enforcer {
    type Error;

    /// Enforce the policy with the provided arguments
    ///
    /// If the enforcement is successful, it should return Ok(true), and
    /// if access is rejected an Ok(false) should be returned.
    ///
    /// On error conditions an error should be returned which is specific
    /// to the implementation.
    fn enforce(&self, agent: &Agent, res: &str, endpoint_group: &str) -> Result<bool, Self::Error>;
}

/// A trait for enforcers derived from [`genpolicy::Policy`]
///
/// This enforcer trait requires the implementation be using the agent
/// and resource provided from the profile to do the enforcement against
/// just the endpoint_group that will be provided for validation.
pub trait GenpolEnforcer {
    type Error;

    /// Enforce the policy with the provided arguments
    ///
    /// If the enforcement is successful, it should return Ok(true), and
    /// if access is rejected an Ok(false) should be returned.
    ///
    /// On error conditions an error should be returned which is specific
    /// to the implementation.
    fn enforce(&self, endpoint_group: &str) -> Result<bool, Self::Error>;
}
