use async_trait::async_trait;
use pmrcore::{
    ac::{
        agent::Agent,
        role::Role,
        traits::PolicyBackend,
        user::User,
        workflow::State,
    },
    error::BackendError,
};

use crate::{
    backend::db::SqliteBackend,
};

async fn grant_role_to_user_sqlite(
    backend: &SqliteBackend,
    user: &User,
    role: Role,
) -> Result<(), BackendError> {
    let role_str = <&'static str>::from(role);
    sqlx::query!(
        r#"
INSERT INTO user_role (
    user_id,
    role
)
VALUES ( ?1, ?2 )
        "#,
        user.id,
        role_str,
    )
    .execute(&*backend.pool)
    .await?
    .last_insert_rowid();
    Ok(())
}

async fn revoke_role_from_user_sqlite(
    backend: &SqliteBackend,
    user: &User,
    role: Role,
) -> Result<(), BackendError> {
    let role_str = <&'static str>::from(role);
    sqlx::query!(
        r#"
DELETE FROM
    user_role
WHERE
    user_id = ?1 AND
    role = ?2
        "#,
        user.id,
        role_str,
    )
    .execute(&*backend.pool)
    .await?;
    Ok(())
}

async fn grant_res_role_to_agent_sqlite(
    backend: &SqliteBackend,
    res: &str,
    agent: &Agent,
    role: Role,
) -> Result<(), BackendError> {
    let user_id: Option<i64> = agent.into();
    let role_str = <&'static str>::from(role);
    sqlx::query!(
        r#"
INSERT INTO res_grant (
    res,
    user_id,
    role
)
VALUES ( ?1, ?2, ?3 )
        "#,
        res,
        user_id,
        role_str,
    )
    .execute(&*backend.pool)
    .await?
    .last_insert_rowid();
    Ok(())
}

async fn revoke_res_role_from_agent_sqlite(
    backend: &SqliteBackend,
    res: &str,
    agent: &Agent,
    role: Role,
) -> Result<(), BackendError> {
    let role_str = <&'static str>::from(role);
    let user_id: Option<i64> = agent.into();
    sqlx::query!(
        r#"
DELETE FROM
    res_grant
WHERE
    res = ?1 AND
    user_id = ?2 AND
    role = ?3
        "#,
        res,
        user_id,
        role_str,
    )
    .execute(&*backend.pool)
    .await?;
    Ok(())
}

async fn assign_policy_to_wf_state_sqlite(
    backend: &SqliteBackend,
    wf_state: State,
    role: Role,
    endpoint_group: &str,
    method: &str,
) -> Result<(), BackendError> {
    let state = <&'static str>::from(wf_state);
    let role = <&'static str>::from(role);
    sqlx::query!(
        r#"
INSERT INTO wf_policy (
    state,
    role,
    endpoint_group,
    method
)
VALUES ( ?1, ?2, ?3, ?4 )
        "#,
        state,
        role,
        endpoint_group,
        method,
    )
    .execute(&*backend.pool)
    .await?
    .last_insert_rowid();
    Ok(())
}

async fn remove_policy_from_wf_state_sqlite(
    backend: &SqliteBackend,
    state: State,
    role: Role,
    endpoint_group: &str,
    method: &str,
) -> Result<(), BackendError> {
    let state = <&'static str>::from(state);
    let role = <&'static str>::from(role);
    sqlx::query!(
        r#"
DELETE FROM
    wf_policy
WHERE
    state = ?1 AND
    role = ?2 AND
    endpoint_group = ?3 AND
    method = ?4
        "#,
        state,
        role,
        endpoint_group,
        method,
    )
    .execute(&*backend.pool)
    .await?;
    Ok(())
}

#[async_trait]
impl PolicyBackend for SqliteBackend {
    async fn grant_role_to_user(
        &self,
        user: &User,
        role: Role,
    ) -> Result<(), BackendError> {
        grant_role_to_user_sqlite(
            &self,
            user,
            role,
        ).await
    }

    async fn revoke_role_from_user(
        &self,
        user: &User,
        role: Role,
    ) -> Result<(), BackendError> {
        revoke_role_from_user_sqlite(
            &self,
            user,
            role,
        ).await
    }

    async fn grant_res_role_to_agent(
        &self,
        res: &str,
        agent: &Agent,
        role: Role,
    ) -> Result<(), BackendError> {
        grant_res_role_to_agent_sqlite(
            &self,
            res,
            agent,
            role,
        ).await
    }

    async fn revoke_res_role_from_agent(
        &self,
        res: &str,
        agent: &Agent,
        role: Role,
    ) -> Result<(), BackendError> {
        revoke_res_role_from_agent_sqlite(
            &self,
            res,
            agent,
            role,
        ).await
    }

    async fn assign_policy_to_wf_state(
        &self,
        wf_state: State,
        role: Role,
        endpoint_group: &str,
        method: &str,
    ) -> Result<(), BackendError> {
        assign_policy_to_wf_state_sqlite(
            &self,
            wf_state,
            role,
            endpoint_group,
            method,
        ).await
    }

    async fn remove_policy_from_wf_state(
        &self,
        wf_state: State,
        role: Role,
        endpoint_group: &str,
        method: &str,
    ) -> Result<(), BackendError> {
        remove_policy_from_wf_state_sqlite(
            &self,
            wf_state,
            role,
            endpoint_group,
            method,
        ).await
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrcore::ac::{
        agent::Agent,
        role::Role,
        traits::{
            PolicyBackend,
            UserBackend,
        },
        workflow::State,
    };
    use crate::backend::db::{
        MigrationProfile,
        SqliteBackend,
    };

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrac)
            .await?;
        let user_id = UserBackend::add_user(&backend, "test_user").await?;
        let agent: Agent = UserBackend::get_user_by_id(&backend, user_id).await?.into();
        let state = State::Published;
        let role = Role::Reader;
        PolicyBackend::grant_res_role_to_agent(&backend, "/", &agent, role).await?;
        PolicyBackend::revoke_res_role_from_agent(&backend, "/", &agent, role).await?;
        PolicyBackend::assign_policy_to_wf_state(&backend, state, role, "", "GET").await?;
        PolicyBackend::remove_policy_from_wf_state(&backend, state, role, "", "GET").await?;
        Ok(())
    }

}
