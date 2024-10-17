use async_trait::async_trait;
use futures::TryStreamExt;
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
use std::{
    collections::HashMap,
    str::FromStr,
};

use crate::{
    backend::db::SqliteBackend,
};

async fn grant_role_to_user_sqlite(
    backend: &SqliteBackend,
    user: &User,
    role: Role,
) -> Result<bool, BackendError> {
    let role_str = <&'static str>::from(role);
    match sqlx::query!(
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
    .await {
        Ok(_) => Ok(true),
        Err(e) => {
            match e.as_database_error() {
                Some(db_e) if db_e.is_unique_violation() => Ok(false),
                _ => Err(e)?,
            }
        }
    }
}

async fn revoke_role_from_user_sqlite(
    backend: &SqliteBackend,
    user: &User,
    role: Role,
) -> Result<bool, BackendError> {
    let role_str = <&'static str>::from(role);
    Ok(sqlx::query!(
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
    .await?
    .rows_affected() > 0)
}

async fn get_roles_for_user_sqlite(
    backend: &SqliteBackend,
    user: &User,
) -> Result<Vec<Role>, BackendError> {
    Ok(sqlx::query!(
        r#"
SELECT
    role
FROM
    user_role
WHERE
    user_id = ?1
        "#,
        user.id,
    )
    .map(|row| Role::from_str(&row.role).unwrap_or(Role::default()))
    .fetch_all(&*backend.pool)
    .await?
    .into())
}

async fn res_grant_role_to_agent_sqlite(
    backend: &SqliteBackend,
    res: &str,
    agent: &Agent,
    role: Role,
) -> Result<bool, BackendError> {
    let user_id: Option<i64> = agent.into();
    let role_str = <&'static str>::from(role);
    match sqlx::query!(
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
    .await {
        Ok(_) => Ok(true),
        Err(e) => {
            match e.as_database_error() {
                Some(db_e) if db_e.is_unique_violation() => Ok(false),
                _ => Err(e)?,
            }
        }
    }
}

async fn res_revoke_role_from_agent_sqlite(
    backend: &SqliteBackend,
    res: &str,
    agent: &Agent,
    role: Role,
) -> Result<bool, BackendError> {
    let role_str = <&'static str>::from(role);
    let user_id: Option<i64> = agent.into();
    Ok(sqlx::query!(
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
    .await?
    .rows_affected() > 0)
}

async fn get_res_grants_for_res_sqlite(
    backend: &SqliteBackend,
    res: &str,
) -> Result<Vec<(Agent, Vec<Role>)>, BackendError> {
    let mut result = HashMap::<Option<i64>, (Agent, Vec<Role>)>::new();
    let mut rows = sqlx::query!(
        r#"
SELECT
    res_grant.user_id AS user_id,
    user.name AS user_name,
    'user'.created_ts as user_created_ts,
    res_grant.role AS role
FROM
    res_grant
LEFT JOIN
    'user' ON res_grant.user_id == 'user'.id
WHERE
    res_grant.res = ?1
        "#,
        res,
    )
    .fetch(&*backend.pool);
    while let Some(row) = rows.try_next().await? {
        result
            .entry(row.user_id)
            .and_modify(|(_, roles)| roles.push(
                Role::from_str(&row.role).unwrap_or(Role::default()),
            ))
            .or_insert((
                match row.user_id {
                    Some(id) => {
                        Agent::User(User {
                            id,
                            name: row.user_name,
                            created_ts: row.user_created_ts,
                        })
                    },
                    _ => Agent::Anonymous,
                },
                vec![Role::from_str(&row.role).unwrap_or(Role::default())],
            ));
    }

    Ok(result.into_values().collect())
}

async fn get_res_grants_for_agent_sqlite(
    backend: &SqliteBackend,
    agent: &Agent,
) -> Result<Vec<(String, Vec<Role>)>, BackendError> {
    let mut result = HashMap::<String, Vec<Role>>::new();
    let user_id: Option<i64> = agent.into();
    let mut rows = sqlx::query!(
        r#"
SELECT
    res,
    role
FROM
    res_grant
WHERE
    res_grant.user_id = ?1
        "#,
        user_id,
    )
    .fetch(&*backend.pool);

    while let Some(row) = rows.try_next().await? {
        result
            .entry(row.res)
            .and_modify(|roles| roles.push(
                Role::from_str(&row.role).unwrap_or(Role::default()),
            ))
            .or_insert(
                vec![Role::from_str(&row.role).unwrap_or(Role::default())],
            );
    }
    Ok(result
        .into_iter()
        .collect::<Vec<_>>())
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
    ) -> Result<bool, BackendError> {
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
    ) -> Result<bool, BackendError> {
        revoke_role_from_user_sqlite(
            &self,
            user,
            role,
        ).await
    }

    async fn get_roles_for_user(
        &self,
        user: &User,
    ) -> Result<Vec<Role>, BackendError> {
        get_roles_for_user_sqlite(
            &self,
            user,
        ).await
    }

    async fn res_grant_role_to_agent(
        &self,
        res: &str,
        agent: &Agent,
        role: Role,
    ) -> Result<bool, BackendError> {
        res_grant_role_to_agent_sqlite(
            &self,
            res,
            agent,
            role,
        ).await
    }

    async fn res_revoke_role_from_agent(
        &self,
        res: &str,
        agent: &Agent,
        role: Role,
    ) -> Result<bool, BackendError> {
        res_revoke_role_from_agent_sqlite(
            &self,
            res,
            agent,
            role,
        ).await
    }

    async fn get_res_grants_for_res(
        &self,
        res: &str,
    ) -> Result<Vec<(Agent, Vec<Role>)>, BackendError> {
        get_res_grants_for_res_sqlite(
            &self,
            res,
        ).await
    }

    async fn get_res_grants_for_agent(
        &self,
        agent: &Agent,
    ) -> Result<Vec<(String, Vec<Role>)>, BackendError> {
        get_res_grants_for_agent_sqlite(
            &self,
            agent,
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
        let user = UserBackend::get_user_by_id(&backend, user_id).await?
            .expect("user is missing?");
        let agent: Agent = user.clone().into();
        let state = State::Published;
        let role = Role::Reader;
        PolicyBackend::res_grant_role_to_agent(&backend, "/", &agent, role).await?;
        assert_eq!(
            vec![(agent.clone(), vec![role])],
            PolicyBackend::get_res_grants_for_res(&backend, "/").await?
        );
        assert_eq!(
            vec![("/".to_string(), vec![role])],
            PolicyBackend::get_res_grants_for_agent(&backend, &agent).await?,
        );
        PolicyBackend::res_revoke_role_from_agent(&backend, "/", &agent, role).await?;
        assert!(PolicyBackend::get_res_grants_for_res(&backend, "/").await?.is_empty());
        assert!(PolicyBackend::get_res_grants_for_agent(&backend, &agent).await?.is_empty());
        PolicyBackend::assign_policy_to_wf_state(&backend, state, role, "", "GET").await?;
        PolicyBackend::remove_policy_from_wf_state(&backend, state, role, "", "GET").await?;

        PolicyBackend::grant_role_to_user(&backend, &user, Role::Manager).await?;
        assert_eq!(
            &[Role::Manager],
            PolicyBackend::get_roles_for_user(&backend, &user).await?.as_slice()
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_double() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrac)
            .await?;
        let user_id = UserBackend::add_user(&backend, "test_user").await?;
        let user = UserBackend::get_user_by_id(&backend, user_id).await?
            .expect("user is missing?");
        let role = Role::Reader;
        assert!(PolicyBackend::grant_role_to_user(&backend, &user, Role::Manager).await?);
        assert!(!PolicyBackend::grant_role_to_user(&backend, &user, Role::Manager).await?);
        assert_eq!(
            &[Role::Manager],
            PolicyBackend::get_roles_for_user(&backend, &user).await?.as_slice()
        );
        assert!(PolicyBackend::revoke_role_from_user(&backend, &user, Role::Manager).await?);
        assert!(PolicyBackend::get_roles_for_user(&backend, &user).await?.is_empty());
        assert!(!PolicyBackend::revoke_role_from_user(&backend, &user, Role::Manager).await?);
        assert!(PolicyBackend::get_roles_for_user(&backend, &user).await?.is_empty());

        let agent = user.into();
        assert!(PolicyBackend::res_grant_role_to_agent(&backend, "/", &agent, role).await?);
        assert!(!PolicyBackend::res_grant_role_to_agent(&backend, "/", &agent, role).await?);
        assert_eq!(
            vec![(agent.clone(), vec![role])],
            PolicyBackend::get_res_grants_for_res(&backend, "/").await?
        );
        assert!(PolicyBackend::res_revoke_role_from_agent(&backend, "/", &agent, role).await?);
        assert!(PolicyBackend::get_res_grants_for_res(&backend, "/").await?.is_empty());
        assert!(!PolicyBackend::res_revoke_role_from_agent(&backend, "/", &agent, role).await?);
        assert!(PolicyBackend::get_res_grants_for_res(&backend, "/").await?.is_empty());

        Ok(())
    }

}
