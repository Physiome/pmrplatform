use crate::{
    ac::traits::SessionBackend,
    chrono::Utc,
    error::BackendError,
};
pub use super::*;

impl dyn SessionBackend {
    pub async fn new_user_session(
        &self,
        token_factory: &SessionTokenFactory,
        user_id: i64,
        origin: String,
    ) -> Result<Session, BackendError> {
        let session = Session::new(token_factory, user_id, origin);
        self.save_session(&session).await?;
        Ok(session)
    }
}

impl Session {
    pub fn new(
        token_factory: &SessionTokenFactory,
        user_id: i64,
        origin: impl Into<String>,
    ) -> Self {
        let origin = origin.into();
        let token = token_factory.create();
        let created_ts = Utc::now().timestamp();
        let last_active_ts = created_ts.clone();
        Self {
            token,
            user_id,
            origin,
            created_ts,
            last_active_ts,
        }
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
        let token_factory = SessionTokenFactory::new();
        let session = Session::new(&token_factory, 1, "localhost");
        assert_eq!(session.user_id, 1);

        set_timestamp(1491625364);
        let token_factory = SessionTokenFactory::new()
            .rng(MockRng::default());
        let session = Session::new(&token_factory, 1, "localhost");
        assert_eq!(session, serde_json::from_str(r#"{
            "token": "9b8a377d5caca2d0b898cf757e46b2af",
            "user_id": 1,
            "origin": "localhost",
            "created_ts": 1491625364,
            "last_active_ts": 1491625364
        }"#)?);
        Ok(())
    }
}
