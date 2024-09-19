use async_trait::async_trait;
use pmrcore::{
    ac::{
        permit::{
            Grant,
            Policy,
            ResourcePolicy,
        },
        traits::ResourceBackend,
        workflow::State,
    },
    error::BackendError,
};

use crate::{
    backend::db::SqliteBackend,
};

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

async fn generate_policy_for_res_sqlite(
    backend: &SqliteBackend,
    res: impl Into<String> + Send,
) -> Result<ResourcePolicy, BackendError> {
    let resource = res.into();
    let res_str = resource.as_str();
    // FIXME Eventually we may need to support all levels of wildcards,
    // so the shortcut `OR res = "/*"` will no longer be sufficient.
    // Well, or whatever way this will be structured.
    let grants = sqlx::query!(
        r#"
SELECT
    res_grant.res as res,
    'user'.name as user,
    res_grant.role AS role
FROM
    res_grant
JOIN
    'user' ON res_grant.user_id == 'user'.id
WHERE
    res = ?1 or res = "/*"
        "#,
        res_str,
    )
    .map(|row| Grant {
        res: row.res,
        user: row.user,
        role: row.role,
    })
    .fetch_all(&*backend.pool)
    .await?;

    let policies = sqlx::query!(
        r#"
SELECT
    wf_policy.role AS role,
    wf_policy.endpoint_group AS endpoint_group,
    wf_policy.method AS method
FROM
    res_wf_state
JOIN
    wf_policy ON res_wf_state.state == wf_policy.state
WHERE
    res = ?1
        "#,
        res_str,
    )
    .map(|row| Policy {
        role: row.role,
        endpoint_group: row.endpoint_group,
        method: row.method,
    })
    .fetch_all(&*backend.pool)
    .await?;

    Ok(ResourcePolicy {
        resource,
        grants,
        policies,
    })
}

#[async_trait]
impl ResourceBackend for SqliteBackend {
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

    async fn generate_policy_for_res(
        &self,
        res: impl Into<String> + Send,
    ) -> Result<ResourcePolicy, BackendError> {
        generate_policy_for_res_sqlite(
            &self,
            res,
        ).await
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrcore::ac::{
        permit::ResourcePolicy,
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
        let policy = ResourceBackend::generate_policy_for_res(&backend, "/").await?;
        assert_eq!(policy, ResourcePolicy {
            resource: "/".to_string(),
            grants: vec![],
            policies: vec![],
        });

        // we only publish here, but no policies/users attached
        let state = State::Published;
        ResourceBackend::set_wf_state_for_res(&backend, "/", state).await?;
        let policy = ResourceBackend::generate_policy_for_res(&backend, "/").await?;
        assert_eq!(policy, ResourcePolicy {
            resource: "/".to_string(),
            grants: vec![],
            policies: vec![],
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
        let user = UserBackend::get_user_by_id(&backend, user_id).await?;
        let state = State::Published;
        let role = Role::Reader;
        PolicyBackend::grant_role_to_user(&backend, "/", &user, role).await?;
        PolicyBackend::assign_policy_to_wf_state(&backend, state, role, "", "GET").await?;
        ResourceBackend::set_wf_state_for_res(&backend, "/", state).await?;

        let policy = ResourceBackend::generate_policy_for_res(&backend, "/").await?;
        assert_eq!(policy, serde_json::from_str(r#"{
            "resource": "/",
            "grants": [
                {"res": "/", "user": "test_user", "role": "Reader"}
            ],
            "policies": [
                {"role": "Reader", "endpoint_group": "", "method": "GET"}
            ]
        }"#)?);

        PolicyBackend::revoke_role_from_user(&backend, "/", &user, role).await?;
        PolicyBackend::remove_policy_from_wf_state(&backend, state, role, "", "GET").await?;
        let policy = ResourceBackend::generate_policy_for_res(&backend, "/").await?;
        assert_eq!(policy, serde_json::from_str(r#"{
            "resource": "/",
            "grants": [
            ],
            "policies": [
            ]
        }"#)?);

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
        let user = UserBackend::get_user_by_id(&backend, user_id).await?;
        let admin_id = UserBackend::add_user(&backend, "admin").await?;
        let admin = UserBackend::get_user_by_id(&backend, admin_id).await?;
        PolicyBackend::grant_role_to_user(
            &backend,
            "/*",
            &admin,
            Role::Manager,
        ).await?;
        PolicyBackend::grant_role_to_user(
            &backend,
            "/item/1",
            &user,
            Role::Owner,
        ).await?;
        PolicyBackend::assign_policy_to_wf_state(
            &backend,
            State::Published,
            Role::Reader,
            "",
            "GET",
        ).await?;
        PolicyBackend::assign_policy_to_wf_state(
            &backend,
            State::Private,
            Role::Owner,
            "edit",
            "POST",
        ).await?;
        PolicyBackend::assign_policy_to_wf_state(
            &backend,
            State::Private,
            Role::Owner,
            "edit",
            "GET",
        ).await?;
        PolicyBackend::assign_policy_to_wf_state(
            &backend,
            State::Published,
            Role::Owner,
            "edit",
            "GET",
        ).await?;

        ResourceBackend::set_wf_state_for_res(
            &backend,
            "/item/1",
            State::Private,
        ).await?;
        let mut policy = ResourceBackend::generate_policy_for_res(&backend, "/item/1").await?;
        policy.grants.sort_unstable();
        policy.policies.sort_unstable();
        assert_eq!(policy, serde_json::from_str(r#"{
            "resource": "/item/1",
            "grants": [
                {"res": "/*", "user": "admin", "role": "Manager"},
                {"res": "/item/1", "user": "test_user", "role": "Owner"}
            ],
            "policies": [
                {"role": "Owner", "endpoint_group": "edit", "method": "GET"},
                {"role": "Owner", "endpoint_group": "edit", "method": "POST"}
            ]
        }"#)?);

        ResourceBackend::set_wf_state_for_res(
            &backend,
            "/item/1",
            State::Published,
        ).await?;
        let mut policy = ResourceBackend::generate_policy_for_res(&backend, "/item/1").await?;
        policy.grants.sort_unstable();
        policy.policies.sort_unstable();
        assert_eq!(policy, serde_json::from_str(r#"{
            "resource": "/item/1",
            "grants": [
                {"res": "/*", "user": "admin", "role": "Manager"},
                {"res": "/item/1", "user": "test_user", "role": "Owner"}
            ],
            "policies": [
                {"role": "Owner", "endpoint_group": "edit", "method": "GET"},
                {"role": "Reader", "endpoint_group": "", "method": "GET"}
            ]
        }"#)?);

        Ok(())
    }

}
