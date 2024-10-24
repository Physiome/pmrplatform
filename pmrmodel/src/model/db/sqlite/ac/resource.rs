use async_trait::async_trait;
use pmrcore::{
    ac::{
        agent::Agent,
        genpolicy::{
            UserRole,
            ResGrant,
            RolePermit,
            Policy,
        },
        role::Role,
        traits::ResourceBackend,
        workflow::State,
    },
    error::BackendError,
};
use std::str::FromStr;

use crate::{
    backend::db::SqliteBackend,
};

async fn get_wf_state_for_res_sqlite(
    backend: &SqliteBackend,
    res: &str,
) -> Result<State, BackendError> {
    let state = sqlx::query!(
        r#"
SELECT
    state
FROM
    res_wf_state
WHERE
    res = ?1
        "#,
        res,
    )
        .map(|row| State::from_str(&row.state).unwrap_or(State::default()))
        .fetch_one(&*backend.pool)
        .await
        .unwrap_or(State::Unknown);
    Ok(state)
}

async fn set_wf_state_for_res_sqlite(
    backend: &SqliteBackend,
    res: &str,
    wf_state: State,
) -> Result<(), BackendError> {
    let state = <&'static str>::from(wf_state);
    sqlx::query!(
        r#"
INSERT INTO res_wf_state (
    res,
    state
)
VALUES (?1, ?2)
ON CONFLICT(res)
DO UPDATE SET
    state = ?2
        "#,
        res,
        state,
    )
    .execute(&*backend.pool)
    .await?;
    Ok(())
}

async fn generate_policy_for_agent_res_sqlite(
    backend: &SqliteBackend,
    agent: &Agent,
    res: impl Into<String> + Send,
) -> Result<Policy, BackendError> {
    let resource = res.into();
    // FIXME Eventually we may need to support all levels of wildcards,
    // so the shortcut `OR res = "/*"` will no longer be sufficient.
    // Well, or whatever way this will be structured.
    let res_str = resource.as_str();

    // note that this explicitly _ignores_ anonymous agents that may have been
    // assigned roles via `user_role` as the schema currently allows null for
    // user_id, but whether we should keep this remains an open question
    let (user_roles, res_grants) = match agent {
        Agent::User(user) => {
            let user_roles = sqlx::query!(
                "\
SELECT
    'user'.name as user,
    user_role.role AS role
FROM
    user_role
JOIN
    'user' ON user_role.user_id == 'user'.id
WHERE
    user_role.user_id = ?1
\
                ",
                user.id,
            )
            .map(|row| UserRole {
                user: row.user,
                role: Role::from_str(&row.role).unwrap_or(Role::default()),
            })
            .fetch_all(&*backend.pool)
            .await?;

            let res_grants = sqlx::query!(
                r#"
SELECT
    res_grant.res as res,
    'user'.name as user_name,
    res_grant.role AS role
FROM
    res_grant
LEFT JOIN
    'user' ON res_grant.user_id == 'user'.id
WHERE
    (res_grant.res = ?1 OR res_grant.res = "/*")
    AND
    (res_grant.user_id == ?2 OR res_grant.user_id is NULL)
        "#,
                res_str,
                user.id,
            )
            .map(|row| ResGrant {
                res: row.res,
                agent: row.user_name,
                role: Role::from_str(&row.role).unwrap_or_default(),
            })
            .fetch_all(&*backend.pool)
            .await?;

            (user_roles, res_grants)
        }
        Agent::Anonymous => {
            let user_roles = vec![];
            let res_grants = sqlx::query!(
                r#"
SELECT
    res_grant.res as res,
    'user'.name as user_name,
    res_grant.role AS role
FROM
    res_grant
LEFT JOIN
    'user' ON res_grant.user_id == 'user'.id
WHERE
    (res = ?1 OR res = "/*")
    AND
    res_grant.user_id is NULL
        "#,
                res_str,
            )
            .map(|row| ResGrant {
                res: row.res,
                agent: row.user_name,
                role: Role::from_str(&row.role).unwrap_or_default(),
            })
            .fetch_all(&*backend.pool)
            .await?;
            (user_roles, res_grants)
        }
    };

    let role_permits = sqlx::query!(
        r#"
SELECT
    wf_policy.role AS role,
    wf_policy.action AS action
FROM
    res_wf_state
JOIN
    wf_policy ON res_wf_state.state == wf_policy.state
WHERE
    res_wf_state.res = ?1
        "#,
        res_str,
    )
    .map(|row| RolePermit {
        role: Role::from_str(&row.role).unwrap_or_default(),
        action: row.action,
    })
    .fetch_all(&*backend.pool)
    .await?;

    Ok(Policy {
        resource,
        user_roles,
        res_grants,
        role_permits,
    })
}

#[async_trait]
impl ResourceBackend for SqliteBackend {
    async fn get_wf_state_for_res(
        &self,
        res: &str,
    ) -> Result<State, BackendError> {
        get_wf_state_for_res_sqlite(
            &self,
            res,
        ).await
    }

    async fn set_wf_state_for_res(
        &self,
        res: &str,
        wf_state: State,
    ) -> Result<(), BackendError> {
        set_wf_state_for_res_sqlite(
            &self,
            res,
            wf_state,
        ).await
    }

    async fn generate_policy_for_agent_res(
        &self,
        agent: &Agent,
        res: String,
    ) -> Result<Policy, BackendError> {
        generate_policy_for_agent_res_sqlite(
            &self,
            agent,
            res,
        ).await
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrcore::ac::{
        agent::Agent,
        genpolicy::Policy,
        role::Role,
        traits::{
            PolicyBackend,
            ResourceBackend,
            UserBackend,
        },
        workflow::State,
    };
    use crate::backend::db::{
        MigrationProfile,
        SqliteBackend,
    };

    #[async_std::test]
    async fn empty() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrac)
            .await?;
        let policy = ResourceBackend::generate_policy_for_agent_res(
            &backend,
            &Agent::Anonymous,
            "/".into(),
        ).await?;
        assert_eq!(policy, Policy {
            resource: "/".to_string(),
            user_roles: vec![],
            res_grants: vec![],
            role_permits: vec![],
        });

        // we only publish here, but no role_permits/users attached
        let state = State::Published;
        ResourceBackend::set_wf_state_for_res(&backend, "/", state).await?;
        let policy = ResourceBackend::generate_policy_for_agent_res(
            &backend,
            &Agent::Anonymous,
            "/".into(),
        ).await?;
        assert_eq!(policy, Policy {
            resource: "/".to_string(),
            user_roles: vec![],
            res_grants: vec![],
            role_permits: vec![],
        });
        Ok(())
    }

    #[async_std::test]
    async fn basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrac)
            .await?;

        let user_id = UserBackend::add_user(&backend, "test_user").await?;
        let user = UserBackend::get_user_by_id(&backend, user_id).await?
            .expect("user is missing?");
        let state = State::Published;
        let role = Role::Reader;
        let agent: Agent = user.clone().into();
        PolicyBackend::grant_role_to_user(&backend, &user, role).await?;
        PolicyBackend::res_grant_role_to_agent(&backend, "/", &agent, role).await?;
        PolicyBackend::assign_policy_to_wf_state(&backend, state, role, "").await?;
        ResourceBackend::set_wf_state_for_res(&backend, "/", state).await?;

        let policy = ResourceBackend::generate_policy_for_agent_res(
            &backend,
            &agent,
            "/".into(),
        ).await?;
        assert_eq!(policy, serde_json::from_str(r#"{
            "resource": "/",
            "user_roles": [
                {"user": "test_user", "role": "Reader"}
            ],
            "res_grants": [
                {"res": "/", "agent": "test_user", "role": "Reader"}
            ],
            "role_permits": [
                {"role": "Reader", "action": ""}
            ]
        }"#)?);

        PolicyBackend::revoke_role_from_user(&backend, &user, role).await?;
        PolicyBackend::res_revoke_role_from_agent(&backend, "/", &agent, role).await?;
        PolicyBackend::remove_policy_from_wf_state(&backend, state, role, "").await?;
        let policy = ResourceBackend::generate_policy_for_agent_res(
            &backend,
            &agent,
            "/".into(),
        ).await?;
        assert_eq!(policy, serde_json::from_str(r#"{
            "resource": "/",
            "user_roles": [
            ],
            "res_grants": [
            ],
            "role_permits": [
            ]
        }"#)?);

        Ok(())
    }

    #[async_std::test]
    async fn anonymous() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrac)
            .await?;
        // we only publish here, but no role_permits/users attached
        let state = State::Published;
        let role = Role::Reader;
        let agent = Agent::Anonymous;
        ResourceBackend::set_wf_state_for_res(&backend, "/", state).await?;
        PolicyBackend::res_grant_role_to_agent(&backend, "/", &agent, role).await?;

        let policy = ResourceBackend::generate_policy_for_agent_res(&backend, &agent, "/".into()).await?;
        assert_eq!(policy, Policy {
            resource: "/".to_string(),
            user_roles: vec![],
            res_grants: serde_json::from_str(r#"[{"res": "/", "user": null, "role": "Reader"}]"#)?,
            role_permits: vec![],
        });
        Ok(())
    }

    #[async_std::test]
    async fn multiple() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrac)
            .await?;

        UserBackend::add_user(&backend, "user321").await?;
        UserBackend::add_user(&backend, "user456").await?;
        let user_id = UserBackend::add_user(&backend, "test_user").await?;
        let user = UserBackend::get_user_by_id(&backend, user_id).await?
            .expect("user is missing?");
        let agent_user: Agent = user.into();
        let admin_id = UserBackend::add_user(&backend, "admin").await?;
        let admin = UserBackend::get_user_by_id(&backend, admin_id).await?
            .expect("user is missing?");
        let agent_admin: Agent = admin.into();
        PolicyBackend::res_grant_role_to_agent(
            &backend,
            "/*",
            &agent_admin,
            Role::Manager,
        ).await?;
        PolicyBackend::res_grant_role_to_agent(
            &backend,
            "/item/1",
            &agent_user,
            Role::Owner,
        ).await?;
        PolicyBackend::assign_policy_to_wf_state(
            &backend,
            State::Published,
            Role::Reader,
            "",
        ).await?;
        PolicyBackend::assign_policy_to_wf_state(
            &backend,
            State::Private,
            Role::Owner,
            "editor_edit",
        ).await?;
        PolicyBackend::assign_policy_to_wf_state(
            &backend,
            State::Private,
            Role::Owner,
            "editor_view",
        ).await?;
        PolicyBackend::assign_policy_to_wf_state(
            &backend,
            State::Published,
            Role::Owner,
            "editor_view",
        ).await?;

        ResourceBackend::set_wf_state_for_res(
            &backend,
            "/item/1",
            State::Private,
        ).await?;
        // TODO should generate policy for agent_user also
        let mut policy = ResourceBackend::generate_policy_for_agent_res(
            &backend,
            &agent_admin,
            "/item/1".into(),
        ).await?;
        policy.res_grants.sort_unstable();
        policy.role_permits.sort_unstable();
        assert_eq!(policy, serde_json::from_str(r#"{
            "resource": "/item/1",
            "user_roles": [
            ],
            "res_grants": [
                {"res": "/*", "agent": "admin", "role": "Manager"}
            ],
            "role_permits": [
                {"role": "Owner", "action": "editor_edit"},
                {"role": "Owner", "action": "editor_view"}
            ]
        }"#)?);

        ResourceBackend::set_wf_state_for_res(
            &backend,
            "/item/1",
            State::Published,
        ).await?;
        let mut policy = ResourceBackend::generate_policy_for_agent_res(
            &backend,
            &agent_admin,
            "/item/1".into(),
        ).await?;
        policy.res_grants.sort_unstable();
        policy.role_permits.sort_unstable();
        assert_eq!(policy, serde_json::from_str(r#"{
            "resource": "/item/1",
            "user_roles": [
            ],
            "res_grants": [
                {"res": "/*", "agent": "admin", "role": "Manager"}
            ],
            "role_permits": [
                {"role": "Owner", "action": "editor_view"},
                {"role": "Reader", "action": ""}
            ]
        }"#)?);

        Ok(())
    }

}
