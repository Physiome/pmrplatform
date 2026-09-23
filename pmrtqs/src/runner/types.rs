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
use tokio_util::{
    sync::CancellationToken,
    task::TaskTracker,
};

use crate::executor::traits;


pub enum RunnerMessage {
    Task(TaskDetached),
    Shutdown,
}

#[derive(Clone)]
pub struct RunnerConf {
    pub poll_until_no_tasks: bool,
}

pub struct Runner<EX: traits::Executor> {
    pub(super) executor: EX,
    pub(super) rt_handle: runtime::Handle,
    pub(super) sender: mpsc::Sender<RunnerMessage>,
    pub(super) receiver: mpsc::Receiver<RunnerMessage>,
    pub(super) semaphore: Arc<Semaphore>,
    pub(super) task_tracker: TaskTracker,
    pub(super) cancellation_token: CancellationToken,
    pub(super) termination_token: Arc<AtomicBool>,
    pub(super) abort_sender: broadcast::Sender<()>,
    pub(super) runner_conf: RunnerConf,
}

#[derive(Clone)]
pub struct RunnerHandle<EX: traits::Executor> {
    pub(super) executor: EX,
    pub(super) abort_sender: broadcast::Sender<()>,
    pub(super) sender: mpsc::Sender<RunnerMessage>,
    pub(super) task_tracker: TaskTracker,
    pub(super) cancellation_token: CancellationToken,
    pub(super) termination_token: Arc<AtomicBool>,
    pub(super) rt_handle: runtime::Handle,
    pub(super) runner_conf: RunnerConf,
}
