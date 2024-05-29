use pmrcore::task::TaskDetached;
use std::{
    sync::{
        Arc,
        atomic::AtomicBool,
    },
};
use tokio::{
    runtime,
    sync::{
        Semaphore,
        broadcast,
        mpsc,
    },
};
use tokio_util::task::TaskTracker;

use crate::executor::traits;


pub enum RunnerMessage {
    Task(TaskDetached),
    Shutdown,
}

pub struct Runner<EX: traits::Executor> {
    pub(super) executor: EX,
    pub(super) rt_handle: runtime::Handle,
    pub(super) sender: mpsc::Sender<RunnerMessage>,
    pub(super) receiver: mpsc::Receiver<RunnerMessage>,
    pub(super) semaphore: Arc<Semaphore>,
    pub(super) task_tracker: TaskTracker,
    pub(super) termination_token: Arc<AtomicBool>,
    pub(super) abort_sender: broadcast::Sender<()>,
}

#[derive(Clone)]
pub struct RunnerHandle<EX: traits::Executor> {
    pub(super) executor: EX,
    pub(super) abort_sender: broadcast::Sender<()>,
    pub(super) sender: mpsc::Sender<RunnerMessage>,
    pub(super) task_tracker: TaskTracker,
    pub(super) termination_token: Arc<AtomicBool>,
    pub(super) rt_handle: tokio::runtime::Handle,
}

pub struct RunnerRuntime<EX: traits::Executor> {
    pub(super) runtime: tokio::runtime::Runtime,
    pub(super) executor: EX,
    pub(super) permits: usize,
    pub(super) handle: Option<RunnerHandle<EX>>,
}
