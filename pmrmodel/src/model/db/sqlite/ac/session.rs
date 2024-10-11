use async_trait::async_trait;
use pmrcore::{
    ac::{
        traits::SessionBackend,
        session::{
            Session,
            SessionToken,
        },
    },
    error::BackendError,
};

use crate::{
    backend::db::SqliteBackend,
    chrono::Utc,
};

async fn save_session_sqlite(
    backend: &SqliteBackend,
    session: &Session,
) -> Result<i64, BackendError> {
    let last_active_ts = Utc::now().timestamp();
    let token_str = session.token.to_string();
    sqlx::query!(
        r#"
INSERT INTO user_session (
    token,
    user_id,
    origin,
    created_ts,
    last_active_ts
)
VALUES ( ?1, ?2, ?3, ?4, ?5 )
ON CONFLICT(token)
DO UPDATE SET
    origin = ?3,
    last_active_ts = ?6
        "#,
        token_str,
        session.user_id,
        session.origin,
        session.created_ts,
        session.last_active_ts,
        last_active_ts,
    )
    .execute(&*backend.pool)
    .await?;
    Ok(last_active_ts)
}

async fn load_session_sqlite(
    backend: &SqliteBackend,
    token: SessionToken,
) -> Result<Session, BackendError> {
    let token_str = token.to_string();
    Ok(sqlx::query!(r#"
SELECT
    token,
    user_id,
    origin,
    created_ts,
    last_active_ts
FROM user_session
WHERE token = ?1
"#,
        token_str,
    )
        .map(|row| Session {
            token,
            user_id: row.user_id,
            origin: row.origin,
            created_ts: row.created_ts,
            last_active_ts: row.last_active_ts,
        })
        .fetch_one(&*backend.pool)
        .await?
    )
}

async fn purge_session_sqlite(
    backend: &SqliteBackend,
    token: SessionToken,
) -> Result<(), BackendError> {
    let token_str = token.to_string();
    sqlx::query!(r#"
DELETE FROM
    user_session
WHERE
    token = ?1
"#,
        token_str,
    )
    .execute(&*backend.pool)
    .await?;
    Ok(())
}

async fn get_user_sessions_sqlite(
    backend: &SqliteBackend,
    user_id: i64,
) -> Result<Vec<Session>, BackendError> {
    Ok(sqlx::query!(r#"
SELECT
    user_id,
    origin,
    created_ts,
    last_active_ts
FROM user_session
WHERE user_id = ?1
"#,
        user_id,
    )
        .map(|row| Session {
            token: SessionToken::default(),
            user_id: row.user_id,
            origin: row.origin,
            created_ts: row.created_ts,
            last_active_ts: row.last_active_ts,
        })
        .fetch_all(&*backend.pool)
        .await?
    )
}

async fn purge_user_sessions_sqlite(
    backend: &SqliteBackend,
    user_id: i64,
    token: Option<SessionToken>,
) -> Result<(), BackendError> {
    let mut builder = sqlx::QueryBuilder::new(r#"
DELETE FROM
    user_session
WHERE user_id = "#);
    builder.push_bind(user_id);
    if let Some(token) = token {
        builder.push("AND token != ");
        builder.push_bind(token.to_string());
    }
    builder.build()
        .execute(&*backend.pool)
        .await?;
    Ok(())
}

#[async_trait]
impl SessionBackend for SqliteBackend {
    async fn save_session(
        &self,
        session: &Session,
    ) -> Result<i64, BackendError> {
        save_session_sqlite(
            &self,
            session,
        ).await
    }

    async fn load_session(
        &self,
        token: SessionToken,
    ) -> Result<Session, BackendError> {
        load_session_sqlite(
            &self,
            token,
        ).await
    }

    async fn purge_session(
        &self,
        token: SessionToken,
    ) -> Result<(), BackendError> {
        purge_session_sqlite(
            &self,
            token,
        ).await
    }

    async fn get_user_sessions(
        &self,
        user_id: i64,
    ) -> Result<Vec<Session>, BackendError> {
        get_user_sessions_sqlite(
            &self,
            user_id,
        ).await
    }

    async fn purge_user_sessions(
        &self,
        user_id: i64,
        token: Option<SessionToken>,
    ) -> Result<(), BackendError> {
        purge_user_sessions_sqlite(
            &self,
            user_id,
            token,
        ).await
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrcore::ac::{
        session::{
            SessionFactory,
            SessionTokenFactory,
        },
        traits::UserBackend,
    };
    use crate::backend::db::{
        MigrationProfile,
        SqliteBackend,
    };
    use test_pmr::{
        chrono::{
            Utc,
            set_timestamp,
        },
        rand::MockRng,
    };

    use super::*;

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrac)
            .await?;
        let user_id = UserBackend::add_user(&backend, "test_user").await?;
        let purge_id = UserBackend::add_user(&backend, "purge_tester").await?;
        let session_factory = SessionFactory::new()
            .token_factory(
                SessionTokenFactory::new()
                    .rng(MockRng::default())
            )
            .ts_source(|| Utc::now().timestamp());
        let session = <dyn SessionBackend>::new_user_session(
            &backend,
            &session_factory,
            user_id,
            "".to_string(),
        ).await?;
        assert_eq!(session.user_id, user_id);

        let stored = SessionBackend::load_session(
            &backend,
            session.token,
        ).await?;
        assert_eq!(session, stored);
        set_timestamp(1888777666);
        let ts = SessionBackend::save_session(
            &backend,
            &stored,
        ).await?;
        assert_eq!(ts, 1888777666);
        let updated = SessionBackend::load_session(
            &backend,
            session.token,
        ).await?;
        assert_eq!(updated.last_active_ts, ts);

        <dyn SessionBackend>::new_user_session(
            &backend,
            &session_factory,
            user_id,
            "".to_string(),
        ).await?;
        let purge = <dyn SessionBackend>::new_user_session(
            &backend,
            &session_factory,
            purge_id,
            "".to_string(),
        ).await?;
        assert_eq!(2, SessionBackend::get_user_sessions(
            &backend,
            user_id,
        ).await?.len());

        let keep = <dyn SessionBackend>::new_user_session(
            &backend,
            &session_factory,
            user_id,
            "".to_string(),
        ).await?;
        let sessions = SessionBackend::get_user_sessions(
            &backend,
            user_id,
        ).await?;
        assert_eq!(3, sessions.len());
        assert_eq!(sessions[0].token, SessionToken::default());
        assert_eq!(sessions[1].token, SessionToken::default());
        assert_eq!(sessions[2].token, SessionToken::default());

        SessionBackend::purge_user_sessions(
            &backend,
            user_id,
            Some(keep.token),
        ).await?;
        assert_eq!(1, SessionBackend::get_user_sessions(
            &backend,
            user_id,
        ).await?.len());

        assert!(SessionBackend::load_session(
            &backend,
            purge.token,
        ).await.is_ok());
        SessionBackend::purge_session(
            &backend,
            purge.token,
        ).await?;
        assert!(SessionBackend::load_session(
            &backend,
            purge.token,
        ).await.is_err());

        SessionBackend::purge_user_sessions(
            &backend,
            user_id,
            None,
        ).await?;
        assert_eq!(0, SessionBackend::get_user_sessions(
            &backend,
            user_id,
        ).await?.len());

        Ok(())
    }

}
