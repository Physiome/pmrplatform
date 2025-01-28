use async_trait::async_trait;
use pmrcore::{
    error::ValueError,
    platform::TMPlatform,
    task::{
       TaskDetached,
       TaskRef,
    },
};
use std::{
    fs::File,
    process::{
        Command,
        Stdio,
    },
};
use tokio::sync::broadcast;

use crate::error::RunnerError;
use super::*;

impl<'a> TMPlatformExecutorInstance<'a> {
    fn new(task: TaskRef<'a>) -> Self {
        Self {
            task,
        }
    }

    pub fn task(&'a self) -> &'a TaskRef<'a> {
        &self.task
    }

    pub async fn execute(&mut self) -> Result<(i32, bool), RunnerError> {
        let mut command: Command = (&self.task).try_into()?;
        let basedir = command.get_current_dir()
            .ok_or(ValueError::UninitializedAttribute("task missing basedir"))?;

        let temp_path = basedir.join("temp");

        std::fs::create_dir_all(&temp_path)?;
        let stdout_file = File::create(temp_path.join("stdout"))?;
        let stderr_file = File::create(temp_path.join("stderr"))?;

        command
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file));

        let mut child = command.spawn()?;
        let pid = child.id();
        // TODO if the DB died here, kill the task?
        self.task.run(pid.into()).await?;
        log::trace!("waiting for child {pid}");
        let exit_status = child.wait()?;
        let code = exit_status.code().unwrap_or(-1);
        log::trace!("child {pid} exit with code {code}");
        self.task.complete(code.into()).await?;
        Ok((code, code == 0))
    }
}

impl<'a> From<TaskRef<'a>> for TMPlatformExecutorInstance<'a> {
    fn from(task: TaskRef<'a>) -> Self {
        Self::new(task)
    }
}

impl<'a> From<TMPlatformExecutorInstance<'a>> for TaskRef<'a> {
    fn from(executor: TMPlatformExecutorInstance<'a>) -> Self {
        executor.task
    }
}

impl<P: Clone + Send + Sync> TMPlatformExecutor<P> {
    pub fn new(platform: P) -> Self {
        Self { platform }
    }
}

#[async_trait]
impl<P: Clone + Send + Sync> traits::Executor for TMPlatformExecutor<P>
where
    P: TMPlatform
{
    type Error = RunnerError;

    async fn start_task(
        &self,
    ) -> Result<Option<TaskDetached>, Self::Error> {
        Ok(self.platform.start_task().await
            .map(|task| task.map(|task| task.detach()))?
        )
    }

    async fn execute(
        &self,
        task: TaskDetached,
        _abort_receiver: broadcast::Receiver<()>,
    ) -> Result<(i32, bool), Self::Error> {
        let mut executor: TMPlatformExecutorInstance = task.bind(&self.platform)?.into();
        // the abort token needs to be passed/run with the
        // executor so it knows if the abort is set.
        executor.execute().await
    }
}
