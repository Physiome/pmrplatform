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
    pub fn new(task: TaskRef<'a, P>) -> Self {
        Self {
            task,
        }
    }

    pub async fn execute(&mut self) -> Result<(), RunnerError> {
        let mut command: Command = (&self.task).try_into()?;
        let temp_path = command.get_current_dir()
            .ok_or(ValueError::UninitializedAttribute("task missing basedir"))?;

        std::fs::create_dir_all(temp_path)?;
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
        Ok(())
    }
}
