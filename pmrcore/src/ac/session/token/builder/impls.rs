use rand::Rng;
use std::sync::Mutex;
pub use super::*;

impl SessionTokenFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rng(mut self, rng: impl RngCore + 'static) -> Self {
        self.rng = Some(Box::new(Mutex::new(rng)));
        self
    }

    pub fn create(&self) -> SessionToken {
        SessionToken(
            self.rng
                .as_ref()
                .map(|m| m.lock()
                    .expect("not poisoned")
                    .gen()
                )
                .unwrap_or_else(|| rand::thread_rng().gen())
        )
    }
}
