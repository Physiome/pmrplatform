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
    path::PathBuf,
    process::{
        Command,
        Stdio,
    },
    sync::Arc,
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
        // the base conversion to command does not handle the creation of directories, but will
        // also join work to the base dir.
        // so, create the temp_path
        let temp_path = PathBuf::from(self.task.basedir()).join("temp");

        // convert the command
        let mut command = Command::try_from(&self.task)?;
        log::trace!("task id {} will run: {command:?}", self.task.id());

        // also create the work path
        let work_path = command.get_current_dir()
            .ok_or(ValueError::UninitializedAttribute("task missing basedir"))?;
        std::fs::create_dir_all(&work_path)?;
        std::fs::create_dir_all(&temp_path)?;

        // and redirect the stdout and stderr to files in temp_path
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

impl TMPlatformExecutor {
    pub fn new(platform: Arc<dyn TMPlatform>) -> Self {
        Self { platform }
    }
}

#[async_trait]
impl traits::Executor for TMPlatformExecutor {
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
        let mut executor: TMPlatformExecutorInstance = task.bind(self.platform.as_ref())?.into();
        // the abort token needs to be passed/run with the
        // executor so it knows if the abort is set.
        executor.execute().await
    }
}
