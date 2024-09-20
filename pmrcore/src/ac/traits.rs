use async_trait::async_trait;
use crate::error::BackendError;
use super::{
    agent::Agent,
    permit::ResourcePolicy,
    role::Role,
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
    ) -> Result<User, BackendError>;
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
    async fn grant_role_to_agent(
        &self,
        res: &str,
        agent: &Agent,
        role: Role,
    ) -> Result<(), BackendError>;
    async fn revoke_role_from_agent(
        &self,
        res: &str,
        agent: &Agent,
        role: Role,
    ) -> Result<(), BackendError>;
    async fn assign_policy_to_wf_state(
        &self,
        wf_state: State,
        role: Role,
        endpoint_group: &str,
        method: &str,
    ) -> Result<(), BackendError>;
    async fn remove_policy_from_wf_state(
        &self,
        wf_state: State,
        role: Role,
        endpoint_group: &str,
        method: &str,
    ) -> Result<(), BackendError>;
}

#[async_trait]
pub trait ResourceBackend {
    async fn set_wf_state_for_res(
        &self,
        res: &str,
        wf_state: State,
    ) -> Result<(), BackendError>;
    async fn generate_policy_for_res(
        &self,
        res: impl Into<String> + Send,
    ) -> Result<ResourcePolicy, BackendError>;
}
