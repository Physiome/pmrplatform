use pmrcore::task::TaskDetached;
use std::{
    sync::{
        Arc,
        atomic::Ordering,
    },
    time::Duration,
};
use tokio::{
    runtime,
    signal,
    sync::{
        Semaphore,
        mpsc,
        broadcast,
    },
    time,
};
use tokio_stream::{
    StreamExt,
    wrappers::IntervalStream,
};
use tokio_util::task::TaskTracker;

use crate::executor::traits;

use super::*;

impl<EX> Runner<EX>
where
    for<'a> EX: traits::Executor + Send + Sync + Clone + 'a,
    <EX as traits::Executor>::Error: Send + std::fmt::Display + std::fmt::Debug
{
    pub fn new(
        executor: EX,
        rt_handle: runtime::Handle,
        permits: usize,  // the number of process permitted
    ) -> Self {
        log::info!("setting up runner with {permits} permits");
        let semaphore = Arc::new(Semaphore::new(permits));
        let task_tracker = TaskTracker::new();
        // not sure if this relative low limit is fine...
        let (sender, receiver) = mpsc::channel(permits);
        let (abort_sender, _) = broadcast::channel(1);
        let termination_token = Arc::new(false.into());
        Self {
            rt_handle,
            sender,
            receiver,
            semaphore,
            task_tracker,
            termination_token,
            abort_sender,
            executor,
        }
    }

    pub fn handle(&self) -> RunnerHandle<EX> {
        RunnerHandle {
            executor: self.executor.clone(),
            sender: self.sender.clone(),
            task_tracker: self.task_tracker.clone(),
            termination_token: self.termination_token.clone(),
            rt_handle: self.rt_handle.clone(),
            abort_sender: self.abort_sender.clone(),
        }
    }

    // run the tasks by starting to listen for the messages
    // this will also listen for shutdown to allow the least intrusive
    // shutdown procedure as this will consume all outstanding task
    // in the receiver queue that was
    pub async fn run(&mut self) {
        log::debug!("runner starting up");
        while let Some(msg) = self.receiver.recv().await {
            if self.termination_token.load(Ordering::Relaxed) {
                self.receiver.close();
                self.task_tracker.close();
                log::debug!("termination token set, ignoring receiver queue");
                break;
            }
            match msg {
                RunnerMessage::Task(task) => {
                    log::debug!("runner received: {task}");
                    let semaphore = self.semaphore.clone();
                    let termination_token = self.termination_token.clone();
                    let executor = self.executor.clone();
                    let abort_receiver = self.abort_sender.subscribe();
                    self.rt_handle.spawn(self.task_tracker.track_future(async move {
                        let t = format!("{task}");
                        // only try to acquire the permit after spawning so this
                        // wouldn't block other messages from being received, like
                        // the shutdown signal
                        let _permit = Arc::clone(&semaphore).acquire_owned().await
                            .expect("semaphore was closed for some reason?");
                        if termination_token.load(Ordering::Relaxed) {
                            log::debug!("runner ignoring task due to termination token: {t}");
                        } else {
                            log::debug!("runner starting task: {t}");
                            // the abort token needs to be passed/run with the
                            // executor so it knows if the abort is set.
                            match executor.execute(task, abort_receiver).await {
                                Ok(_) => (),
                                Err(e) => log::error!("task executor error: {e}"),
                            }
                            log::debug!("runner finished task: {t}")
                        }
                    }));
                },
                RunnerMessage::Shutdown => {
                    self.receiver.close();
                    self.task_tracker.close();
                    log::debug!("runner shutdown signal received");
                },
            }
        }
        log::debug!("runner shutting down");
    }
}

impl<EX> RunnerHandle<EX>
where
    for<'a> EX: traits::Executor + Sync + Send + Clone + 'a,
    <EX as traits::Executor>::Error: Send + std::fmt::Display + std::fmt::Debug
{
    // queue_task sends a message through the sender which hopefully the
    // underlying runner will receive and do something with it.
    pub async fn queue_task(&self, task: TaskDetached) {
        match self.sender.send(RunnerMessage::Task(task)).await {
            Ok(()) => (),
            Err(_) => log::debug!("failed to queue new task to runner as it is no longer listening"),
        }
    }

    pub async fn shutdown(&self) {
        match self.sender.send(RunnerMessage::Shutdown).await {
            Ok(()) => (),
            Err(_) => log::debug!("failed to send shutdown signal to runner as it's no longer listening."),
        }
        log::debug!("waiting for task_tracker...");
        self.task_tracker.wait().await;
        log::debug!("finished waiting for task_tracker");
    }

    pub fn abort(&self) {
        match self.abort_sender.send(()) {
            Ok(n) => log::debug!("abort signal sent to {n} receiver(s)."),
            Err(_) => log::debug!("abort sent but there are no receivers."),
        }
    }

    pub fn terminate(&self) {
        self.termination_token.store(true, Ordering::Relaxed);
    }

    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }

    // poll tasks repeatedly and queue it
    pub async fn poll(&self) {
        let mut ticker = IntervalStream::new(time::interval(Duration::from_millis(100)));
        log::debug!("task queue starting");
        while let Some(_) = (!self.is_closed()).then_some(ticker.next().await).flatten() {
            while let Some(task) = self.executor.start_task()
                .await
                // FIXME need to figure out a more robust way to deal
                // with the following error, as database crash will
                // probably crash the task polling completely.
                .expect("database error when trying to poll for a new task")
            {
                log::debug!("sending task {task}");
                self.queue_task(task).await;
            };
        };
        log::debug!("task queue stopping");
    }

    pub async fn wait_for_abort_signal(&self) {
        log::trace!("waiting for abort signal");
        match signal::ctrl_c().await {
            Ok(()) => {
                log::debug!("Ctrl-C received for abort");
                self.abort();
                log::debug!("abort broadcast set");
            },
            Err(err) => {
                log::debug!("Unable to listen for shutdown signal: {}", err);
                log::debug!("abort not signaled");
            },
        }
    }

    pub async fn wait_for_terminate_signal(&self) {
        log::trace!("waiting for termination signal");
        match signal::ctrl_c().await {
            Ok(()) => {
                log::debug!("Ctrl-C received for terminate");
                let handle = self.clone();
                self.rt_handle.spawn({async move {
                    handle.wait_for_abort_signal().await;
                }});
                self.terminate();
                log::debug!("termination token set");
            },
            Err(err) => {
                log::debug!("Unable to listen for shutdown signal: {}", err);
                log::debug!("terminate not signaled");
            },
        }
    }

    // FIXME somehow these ctrl-c is not fully trapped and is leaked to
    // the subprocess...
    pub async fn wait_for_shutdown_signal(&self) {
        log::trace!("waiting for shutdown signal");
        match signal::ctrl_c().await {
            Ok(()) => {
                log::debug!("Ctrl-C received for shutdown");
                let handle = self.clone();
                self.rt_handle.spawn({async move {
                    handle.wait_for_terminate_signal().await;
                }});
                self.shutdown().await;
                log::debug!("termination confirmed");
            },
            Err(err) => {
                log::debug!("Unable to listen for shutdown signal: {}", err);
                log::debug!("shutdown not signaled");
            },
        }
    }
}

impl<EX> RunnerRuntime<EX>
where
    for<'a> EX: traits::Executor + Send + Sync + Clone + 'a,
    <EX as traits::Executor>::Error: Send + std::fmt::Display + std::fmt::Debug
{
    pub fn new(
        executor: EX,
        permits: usize,
    ) -> Self {
        let runtime = runtime::Builder::new_multi_thread()
            .enable_io()
            .enable_time()
            .build()
            .expect("unable to create the runner runtime");
        Self {
            runtime,
            executor,
            permits,
            handle: None,
        }
    }

    pub fn start(&mut self) {
        if self.handle.is_some() {
            return () // don't start again
        }

        let mut runner: Runner<EX> = Runner::new(
            self.executor.clone(),
            self.runtime.handle().clone(),
            self.permits,
        );
        let runner_handle = runner.handle();
        self.handle = Some(runner_handle.clone());
        self.runtime.spawn({async move {
            runner_handle.poll().await
        }});
        self.runtime.spawn({async move {
            runner.run().await
        }});
    }

    pub fn wait(&mut self) {
        // do nothing if not started before
        if let Some(handle) = &self.handle {
            self.runtime.block_on(async {
                handle.wait_for_shutdown_signal().await;
            });
        }
    }
}
