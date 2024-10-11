use crate::chrono::Utc;
use super::*;

impl SessionFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn token_factory(mut self, val: SessionTokenFactory) -> Self {
        self.token_factory = val;
        self
    }

    pub fn ts_source(mut self, val: impl Fn() -> i64 + Send + 'static) -> Self {
        self.ts_source = Some(Box::new(val));
        self
    }

    pub fn create(
        &self,
        user_id: i64,
        origin: impl Into<String>,
    ) -> Session {
        let origin = origin.into();
        let token = self.token_factory.create();
        let created_ts = self.ts_source
            .as_ref()
            .map(|f| f())
            .unwrap_or_else(|| Utc::now().timestamp());
        let last_active_ts = created_ts;
        Session {
            token,
            user_id,
            origin,
            created_ts,
            last_active_ts,
        }
    }
}
