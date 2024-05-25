use pmrcore::task::TaskDetached;
use pmrmodel::backend::db::Backend;
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
        mpsc,
    },
};
use tokio_util::task::TaskTracker;


pub enum RunnerMessage {
    Task(TaskDetached),
    Shutdown,
}

pub struct Runner<DB> {
    pub(super) backend: Backend<DB>,
    pub(super) rt_handle: runtime::Handle,
    pub(super) sender: mpsc::Sender<RunnerMessage>,
    pub(super) receiver: mpsc::Receiver<RunnerMessage>,
    pub(super) semaphore: Arc<Semaphore>,
    pub(super) task_tracker: TaskTracker,
    pub(super) termination_token: Arc<AtomicBool>,
    // TODO these are reserved for the use of aborting a running process
    pub(super) _abort_sender: mpsc::Sender<()>,
    pub(super) _abort_receiver: mpsc::Receiver<()>,
}

#[derive(Clone)]
pub struct RunnerHandle<DB> {
    pub(super) backend: Backend<DB>,
    pub(super) _abort_sender: mpsc::Sender<()>,
    pub(super) sender: mpsc::Sender<RunnerMessage>,
    pub(super) task_tracker: TaskTracker,
    pub(super) termination_token: Arc<AtomicBool>,
    pub(super) rt_handle: tokio::runtime::Handle,
}

pub struct RunnerRuntime<DB> {
    pub(super) runtime: tokio::runtime::Runtime,
    pub(super) backend: Backend<DB>,
    pub(super) permits: usize,
    pub(super) handle: Option<RunnerHandle<DB>>,
}
