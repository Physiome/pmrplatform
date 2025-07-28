use async_trait::async_trait;
use pmrcore::{
    ac::{
        traits::UserBackend,
        user::User,
    },
    error::BackendError,
};

use crate::{
    SqliteBackend,
    chrono::Utc,
};

async fn add_user_sqlite(
    backend: &SqliteBackend,
    name: &str,
) -> Result<i64, BackendError> {
    let ts = Utc::now().timestamp();
    let id = sqlx::query!(
        r#"
INSERT INTO 'user' (
    name,
    created_ts
)
VALUES ( ?1, ?2 )
        "#,
        name,
        ts,
    )
    .execute(&*backend.pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

async fn get_user_by_id_sqlite(
    backend: &SqliteBackend,
    id: i64,
) -> Result<Option<User>, BackendError> {
    let recs = sqlx::query!(r#"
SELECT
    id,
    name,
    created_ts
FROM
    'user'
WHERE
    id = ?1
        "#,
        id,
    )
    .map(|row| User {
        id: row.id,
        name: row.name,
        created_ts: row.created_ts,
    })
    .fetch_optional(&*backend.pool)
    .await?;
    Ok(recs)
}

async fn get_user_by_name_sqlite(
    backend: &SqliteBackend,
    name: &str,
) -> Result<Option<User>, BackendError> {
    let recs = sqlx::query!(r#"
SELECT
    id,
    name,
    created_ts
FROM
    'user'
WHERE
    name = ?1
        "#,
        name,
    )
    .map(|row| User {
        id: row.id,
        name: row.name,
        created_ts: row.created_ts,
    })
    .fetch_optional(&*backend.pool)
    .await?;
    Ok(recs)
}

async fn store_user_password_sqlite(
    backend: &SqliteBackend,
    user_id: i64,
    password: &str,
) -> Result<i64, BackendError> {
    let ts = Utc::now().timestamp();
    let id = sqlx::query!(
        r#"
INSERT INTO user_password (
    user_id,
    password,
    created_ts
)
VALUES ( ?1, ?2, ?3 )
        "#,
        user_id,
        password,
        ts,
    )
    .execute(&*backend.pool)
    .await?
    .last_insert_rowid();
    Ok(id)
}

async fn get_user_password_sqlite(
    backend: &SqliteBackend,
    user_id: i64,
) -> Result<String, BackendError> {
    let result = sqlx::query!(
        r#"
SELECT
    password
FROM
    user_password
WHERE
    user_id = ?1
ORDER BY id DESC
        "#,
        user_id,
    )
    .map(|row| row.password)
    .fetch_one(&*backend.pool)
    .await?;
    Ok(result)
}

async fn purge_user_passwords_sqlite(
    backend: &SqliteBackend,
    user_id: i64,
) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
DELETE FROM
    user_password
WHERE
    user_id = ?1
        "#,
        user_id,
    )
    .execute(&*backend.pool)
    .await?;
    Ok(())
}


#[async_trait]
impl UserBackend for SqliteBackend {
    async fn add_user(
        &self,
        name: &str,
    ) -> Result<i64, BackendError> {
        add_user_sqlite(
            &self,
            name,
        ).await
    }

    async fn get_user_by_id(
        &self,
        id: i64,
    ) -> Result<Option<User>, BackendError> {
        get_user_by_id_sqlite(
            &self,
            id,
        ).await
    }

    async fn get_user_by_name(
        &self,
        name: &str,
    ) -> Result<Option<User>, BackendError> {
        get_user_by_name_sqlite(
            &self,
            name,
        ).await
    }

    async fn store_user_password(
        &self,
        user_id: i64,
        password: &str,
    ) -> Result<i64, BackendError> {
        store_user_password_sqlite(
            &self,
            user_id,
            password,
        ).await
    }

    async fn get_user_password(
        &self,
        user_id: i64,
    ) -> Result<String, BackendError> {
        get_user_password_sqlite(
            &self,
            user_id,
        ).await
    }

    async fn purge_user_passwords(
        &self,
        user_id: i64,
    ) -> Result<(), BackendError> {
        purge_user_passwords_sqlite(
            &self,
            user_id,
        ).await
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrcore::{
        platform::PlatformConnector as _,
        ac::{
            traits::UserBackend,
            user::User,
        },
    };
    use crate::SqliteBackend;
    use test_pmr::chrono::set_timestamp;

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::ac("sqlite::memory:".into())
            .await
            .map_err(anyhow::Error::from_boxed)?;
        let user_id = UserBackend::add_user(&backend, "test_user").await?;
        let user = UserBackend::get_user_by_id(&backend, user_id).await?
            .expect("user is missing?");
        assert_eq!(
            user,
            User {
                id: 1,
                name: "test_user".to_string(),
                created_ts: 1234567890,
            },
        );

        // note that these tests **ONLY** test for the storage and retrieval of passwords,
        // and has absolutely nothing to do with the security of the incoming passwords
        // before getting to this API call.
        set_timestamp(0);
        UserBackend::store_user_password(&backend, user_id, "password1").await?;
        set_timestamp(1234567899);
        UserBackend::store_user_password(&backend, user_id, "password2").await?;
        assert_eq!(UserBackend::get_user_password(&backend, user_id).await?, "password2");

        UserBackend::purge_user_passwords(&backend, user_id).await?;
        assert!(UserBackend::get_user_password(&backend, user_id).await.is_err());

        UserBackend::store_user_password(&backend, user_id, "password1").await?;
        UserBackend::store_user_password(&backend, user_id, "password2").await?;
        UserBackend::store_user_password(&backend, user_id, "password3").await?;
        assert_eq!(UserBackend::get_user_password(&backend, user_id).await?, "password3");

        Ok(())
    }

}
