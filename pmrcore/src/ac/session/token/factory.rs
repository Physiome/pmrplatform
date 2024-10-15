use rand::RngCore;
use std::sync::Mutex;
pub use super::*;

#[derive(Default)]
pub struct SessionTokenFactory {
    pub(super) rng: Option<Box<Mutex<dyn RngCore + Send + Sync>>>,
}

mod impls;
