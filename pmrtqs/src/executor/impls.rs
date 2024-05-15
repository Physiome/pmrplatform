use pmrcore::{
    error::ValueError,
    platform::TMPlatform,
    task::TaskRef,
};
use std::{
    fs::File,
    process::{
        Command,
        Stdio,
    },
};

use crate::error::RunnerError;
use super::*;

impl<'a, P: TMPlatform + Sync> Executor<'a, P> {
    fn new(task: TaskRef<'a, P>) -> Self {
        Self {
            task,
        }
    }

    pub fn task(&'a self) -> &TaskRef<'a, P> {
        &self.task
    }

    pub async fn execute(&mut self) -> Result<i32, RunnerError> {
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
        let exit_status = child.wait()?;
        let code = exit_status.code().unwrap_or(-1);
        self.task.complete(code.into()).await?;
        Ok(code)
    }
}

impl<'a, P: TMPlatform + Sync> From<TaskRef<'a, P>> for Executor<'a, P> {
    fn from(task: TaskRef<'a, P>) -> Self {
        Self::new(task)
    }
}

impl<'a, P: TMPlatform + Sync> From<Executor<'a, P>> for TaskRef<'a, P> {
    fn from(executor: Executor<'a, P>) -> Self {
        executor.task
    }
}
