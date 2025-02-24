use tokio::runtime;

use crate::{
    executor::traits,
    runner::Runner,
};

use super::*;

impl<EX> From<EX> for Builder<EX>
where
    for<'a> EX: traits::Executor + Send + Sync + Clone + 'a,
    <EX as traits::Executor>::Error: Send + std::fmt::Display + std::fmt::Debug
{
    fn from(value: EX) -> Self {
        Self::new()
            .executor(value)
    }
}

impl<EX> Builder<EX>
where
    for<'a> EX: traits::Executor + Send + Sync + Clone + 'a,
    <EX as traits::Executor>::Error: Send + std::fmt::Display + std::fmt::Debug
{
    pub fn new() -> Self {
        Self {
            executor: None,
            permits: 0,
        }
    }

    pub fn executor(mut self, value: EX) -> Self {
        self.executor = Some(value);
        self
    }

    pub fn permits(mut self, value: usize) -> Self {
        self.permits = value;
        self
    }

    pub fn build(self) -> Runtime<EX> {
        Runtime::new(
            self.executor
                .expect("Executor was not provided with Builder"),
            self.permits,
        )
    }

    pub fn build_with_handle(
        self,
        handle: runtime::Handle,
    ) -> Runtime<EX> {
        Runtime::with_handle(
            handle,
            self.executor
                .expect("Executor was not provided with Builder"),
            self.permits,
        )
    }
}

impl<EX> Runtime<EX>
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
        let handle = runtime.handle().clone();
        Self {
            runtime: Some(runtime),
            handle,
            executor,
            permits,
            driver: None,
        }
    }

    pub fn with_handle(
        handle: runtime::Handle,
        executor: EX,
        permits: usize,
    ) -> Self {
        Self {
            runtime: None,
            handle,
            executor,
            permits,
            driver: None,
        }
    }

    pub fn start(&mut self) {
        if self.driver.is_some() {
            return () // don't start again
        }

        let mut runner: Runner<EX> = Runner::new(
            self.executor.clone(),
            self.handle.clone(),
            self.permits,
        );
        let runner_handle = runner.handle();
        self.driver = Some(runner_handle.clone());
        self.handle.spawn({async move {
            runner_handle.poll().await
        }});
        self.handle.spawn({async move {
            runner.run().await
        }});
    }

    pub fn wait(&mut self) {
        if let Some(runtime) = &self.runtime {
            runtime.block_on(async {
                self.shutdown_signal().await
            });
        } else {
            self.handle.block_on(async {
                self.shutdown_signal().await
            });
        }
    }

    pub async fn shutdown_signal(&self) {
        // do nothing if not started before
        if let Some(driver) = &self.driver {
            driver.wait_for_shutdown_signal().await;
        }
    }
}
