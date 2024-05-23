use clap::Parser;
use futures::StreamExt;
use pmrcore::{
    platform::TMPlatform,
    task::TaskDetached,
};
use pmrmodel::backend::db::{
    Profile,
    Backend,
    SqliteBackend,
};
use pmrtqs::{
    error::RunnerError,
    executor::Executor,
};
use tokio::{
    self,
    runtime,
    time,
};
use tokio_stream::wrappers::IntervalStream;
use sqlx::{
    Sqlite,
    migrate::MigrateDatabase,
};
use std::{
    thread,
    time::Duration,
    sync::Arc,
};

// the runner may become a separate sync function
// start the long running command
// - it will wait for 1 second? at a time
// - check the cancellation token, if not cancelled, keep going
//   - if cancelled, reduce life to 10 seconds.
// - repeat 600 times (5 minutes?)
// - kill the command if still running after that

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short = 'r', long = "runners", default_value = "8")]
    runners: usize,
    #[clap(long, value_name = "PMR_DATA_ROOT", env = "PMR_DATA_ROOT")]
    pmr_data_root: String,
    #[clap(long, value_name = "PMR_REPO_ROOT", env = "PMR_REPO_ROOT")]
    pmr_repo_root: String,
    #[clap(long, value_name = "PMRAPP_DB_URL", env = "PMRAPP_DB_URL")]
    pmrapp_db_url: String,
    #[clap(long, value_name = "PMRTQS_DB_URL", env = "PMRTQS_DB_URL")]
    pmrtqs_db_url: String,
    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    verbose: u8,
}

pub struct Runner<DB> {
    backend: Backend<DB>,
    semaphore: Arc<tokio::sync::Semaphore>,
    receiver: tokio::sync::mpsc::Receiver<TaskDetached>,
    rt_handle: tokio::runtime::Handle,
}

impl<DB> Runner<DB>
where
    Backend<DB>: TMPlatform,
    for<'a> DB: Sync + Send + Clone + 'a
{
    pub fn new(
        backend: Backend<DB>,
        receiver: tokio::sync::mpsc::Receiver<TaskDetached>,
        rt_handle: tokio::runtime::Handle,
        runners: usize,
    ) -> Self {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(runners));
        Self {
            backend,
            semaphore,
            receiver,
            rt_handle,
        }
    }

    pub async fn run(&mut self) {
        // FIXME this doesn't actually block new tasks over the runner limit...
        // the prototype kind of did but it was a simplified model but also
        // didn't involve this message passing...
        while let Some(task) = self.receiver.recv().await {
            log::debug!("runner received task: {}", task.id());
            let handle = self.rt_handle.clone();
            let backend = self.backend.clone();
            let permit = Arc::clone(&self.semaphore).acquire_owned().await;
            handle.spawn(async move {
                let id = task.id();
                let _permit = permit;
                log::debug!("runner starting task: {id}");
                // hold onto the permit until this is done.
                let mut executor: Executor<Backend<DB>> = task.bind(&backend)
                    .expect("task isn't part of this backend")
                    .into();
                executor.execute().await
                    .expect("job failed");
                log::debug!("runner finished task: {id}")
            });
        }
    }
}

#[async_std::main]
async fn main() -> Result<(), RunnerError> {
    dotenvy::dotenv().ok();
    let args = Cli::parse();
    stderrlog::new()
        .module(module_path!())
        .verbosity((args.verbose as usize) + 1)
        .timestamp(stderrlog::Timestamp::Second)
        .init()
        .unwrap();

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let (task_tx, task_rx) = tokio::sync::mpsc::channel(args.runners);
    let backend = SqliteBackend::from_url(&args.pmrtqs_db_url)
        .await
        .map_err(pmrcore::error::BackendError::from)?
        .run_migration_profile(Profile::Pmrtqs)
        .await
        .map_err(pmrcore::error::BackendError::from)?;
    let runner_backend = backend.clone();

    let runner_thread = thread::spawn(move || {
        let runtime = runtime::Builder::new_multi_thread()
            .worker_threads(args.runners)
            .enable_time()
            .build()
            .expect("unable to create the runner runtime");

        let mut runner = Runner::new(
            runner_backend,
            task_rx,
            runtime.handle().clone(),
            args.runners,
        );

        runtime.handle().spawn({
            async move { runner.run().await }
        });

        log::debug!("starting runner runtime");
        // Continue running until notified to shutdown
        runtime.block_on(async {
            shutdown_rx.await.expect("error with shutdown channel");
        });
        log::debug!("runner runtime terminated");
    });

    let listener_thread = thread::spawn(move || {

        let runtime = runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("unable to create the listener runtime");

        log::debug!("starting listener runtime");
        runtime.block_on(async {
            log::debug!("polling for new tasks...");

            if !Sqlite::database_exists(&args.pmrtqs_db_url).await.unwrap_or(false) {
                log::warn!("database {} does not exist; creating...", &args.pmrtqs_db_url);
                Sqlite::create_database(&args.pmrtqs_db_url).await
                    .map_err(pmrcore::error::BackendError::from)?
            }
            let mut ticker = IntervalStream::new(time::interval(Duration::from_millis(250)));

            while let Some(_) = ticker.next().await {
                while let Some(task) = backend.start_task().await?.map(|task| task.detach()) {
                    log::debug!("got task {}", task.id());
                    task_tx.send(task).await
                        .expect("unable to send task to runner thread");
                };
            };
            Ok::<(), RunnerError>(())
        })?;
        log::debug!("listener runtime terminated");
        Ok::<(), RunnerError>(())
    });

    listener_thread.join()
        .expect("listener thread panicked")?;
    shutdown_tx.send(())
        .expect("unable to shutdown runner thread");
    runner_thread.join()
        .expect("runner thread panicked");

    log::debug!("shutting down");
    Ok(())
}
