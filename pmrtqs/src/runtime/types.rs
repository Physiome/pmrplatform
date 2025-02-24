use tokio::runtime::Handle;

use crate::{
    executor::traits,
    runner::RunnerHandle,
};

#[derive(Default)]
pub struct Builder<EX: traits::Executor> {
    pub(super) executor: Option<EX>,
    pub(super) permits: usize,
}

pub struct Runtime<EX: traits::Executor> {
    pub(super) runtime: Option<tokio::runtime::Runtime>,
    pub(super) handle: Handle,
    pub(super) executor: EX,
    pub(super) permits: usize,
    pub(super) driver: Option<RunnerHandle<EX>>,
}
