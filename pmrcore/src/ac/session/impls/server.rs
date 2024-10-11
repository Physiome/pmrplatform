use crate::{
    ac::traits::SessionBackend,
    error::BackendError,
};
pub use super::*;

impl dyn SessionBackend {
    pub async fn new_user_session(
        &self,
        session_factory: &SessionFactory,
        user_id: i64,
        origin: String,
    ) -> Result<Session, BackendError> {
        let session = session_factory.create(user_id, origin);
        self.save_session(&session).await?;
        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use test_pmr::{
        chrono::set_timestamp,
        rand::MockRng,
    };
    use super::*;

    #[test]
    fn gen_token() -> anyhow::Result<()> {
        let factory = SessionTokenFactory::new();
        let token = factory.create();
        let s = token.to_string();
        let p = SessionToken::from_str(&s)?;
        assert_eq!(token, p);
        Ok(())
    }

    #[test]
    fn gen_session() -> anyhow::Result<()> {
        let session_factory = SessionFactory::new();
        let session = session_factory.create(1, "localhost");
        assert_eq!(session.user_id, 1);

        set_timestamp(1491625364);
        let session_factory = SessionFactory::new()
            .token_factory(
                SessionTokenFactory::new()
                    .rng(MockRng::default())
            );
        let session = session_factory.create(1, "localhost");
        assert_eq!(session, serde_json::from_str(r#"{
            "token": "9b8a377d5caca2d0b898cf757e46b2af",
            "user_id": 1,
            "origin": "localhost",
            "created_ts": 1491625364,
            "last_active_ts": 1491625364
        }"#)?);

        let session_factory = SessionFactory::new()
            .token_factory(
                SessionTokenFactory::new()
                    .rng(MockRng::default())
            )
            .ts_source(|| 123);
        let session = session_factory.create(1, "localhost");
        assert_eq!(session, serde_json::from_str(r#"{
            "token": "9b8a377d5caca2d0b898cf757e46b2af",
            "user_id": 1,
            "origin": "localhost",
            "created_ts": 123,
            "last_active_ts": 123
        }"#)?);
        Ok(())
    }
}
