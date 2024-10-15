use super::*;

#[derive(Default)]
pub struct SessionFactory {
    pub(super) token_factory: SessionTokenFactory,
    pub(super) ts_source: Option<Box<dyn Fn() -> i64 + Send + Sync + 'static>>,
}

mod impls;
