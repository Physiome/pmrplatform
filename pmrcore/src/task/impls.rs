use std::{
    ops::Deref,
    path::PathBuf,
    process,
};
use crate::{
    error::ValueError,
    task::*,
};

impl From<[&str; 2]> for TaskArg {
    fn from(s: [&str; 2]) -> Self {
        Self { arg: s[0].to_owned() + s[1], .. Default::default() }
    }
}

impl From<&str> for TaskArg {
    fn from(s: &str) -> Self {
        Self { arg: s.into(), .. Default::default() }
    }
}

impl From<Vec<TaskArg>> for TaskArgs {
    fn from(args: Vec<TaskArg>) -> Self {
        Self(args)
    }
}

impl Deref for TaskArgs {
    type Target = Vec<TaskArg>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<&'a TaskArgs> for Vec<&'a str> {
    fn from(task_args: &'a TaskArgs) -> Self {
        task_args.iter()
            .map(|task_arg| task_arg.arg.as_ref())
            .collect()
    }
}

impl TryFrom<&Task> for process::Command {
    type Error = ValueError;

    fn try_from(task: &Task) -> Result<Self, Self::Error> {
        let mut cmd = process::Command::new(&task.bin_path);
        cmd
            .args::<Vec<&str>, &str>(
                task.args
                    .as_ref()
                    .ok_or(ValueError::Uninitialized)?
                    .into()
            )
            .current_dir(PathBuf::from(&task.basedir).join("work"));
        Ok(cmd)
    }
}

impl TryFrom<&TaskRef<'_>> for process::Command {
    type Error = ValueError;

    fn try_from(task: &TaskRef) -> Result<Self, Self::Error> {
        (&task.inner).try_into()
    }
}

#[cfg(test)]
pub(crate) mod test {
    use std::process::Command;
    use tempfile::TempDir;
    use crate::task::Task;
    use test_binary::build_test_binary_once;

    #[derive(serde::Deserialize)]
    pub(crate) struct Sentinel {
        pub(crate) args: Vec<String>,
        pub(crate) cwd: String,
    }

    #[test]
    fn test_command() -> anyhow::Result<()> {
        // FIXME platform specific pathsep
        build_test_binary_once!(sentinel, "../testing");
        let bin_path = path_to_sentinel().into_string().expect("valid string");
        let tempdir = TempDir::new()?;
        let workdir = tempdir.path().join("work");
        std::fs::create_dir_all(&workdir)?;
        let task = Task {
            bin_path: bin_path.clone(),
            args: Some(vec!["hello".into(), "world".into()].into()),
            basedir: tempdir.path().to_str().expect("valid utf8").to_string(),
            .. Default::default()
        };
        let mut cmd: Command = (&task).try_into()?;
        let output = String::from_utf8(cmd.output()?.stdout)?;
        let result: Sentinel = serde_json::from_str(&output)?;
        assert_eq!(result.args.as_slice(), &[bin_path.as_ref(), "hello", "world"]);
        assert_eq!(result.cwd, tempdir.path().join("work").to_str().expect("valid utf-8").to_string());
        Ok(())
    }
}

#[cfg(feature = "tokio")]
mod tokio_impls {
    use tokio::process::Command;
    use super::*;

    impl TryFrom<&Task> for Command {
        type Error = ValueError;

        fn try_from(task: &Task) -> Result<Self, Self::Error> {
            let mut cmd = Command::new(&task.bin_path);
            cmd
                .args::<Vec<&str>, &str>(
                    task.args
                        .as_ref()
                        .ok_or(ValueError::Uninitialized)?
                        .into()
                )
                .current_dir(PathBuf::from(&task.basedir).join("work"));
            Ok(cmd)
        }
    }

    impl TryFrom<&TaskRef<'_>> for Command {
        type Error = ValueError;

        fn try_from(task: &TaskRef) -> Result<Self, Self::Error> {
            (&task.inner).try_into()
        }
    }

    #[cfg(test)]
    pub(crate) mod test {
        use tempfile::TempDir;
        use tokio::process::Command;
        use crate::task::{
            Task,
            impls::test::Sentinel,
        };
        use test_binary::build_test_binary_once;

        #[tokio::test]
        async fn test_tokio_command() -> anyhow::Result<()> {
            // FIXME platform specific pathsep
            build_test_binary_once!(sentinel, "../testing");
            let bin_path = path_to_sentinel().into_string().expect("valid string");
            let tempdir = TempDir::new()?;
            let workdir = tempdir.path().join("work");
            std::fs::create_dir_all(&workdir)?;
            let task = Task {
                bin_path: bin_path.clone(),
                args: Some(vec!["hello".into(), "world".into()].into()),
                basedir: tempdir.path().to_str().expect("valid utf8").to_string(),
                .. Default::default()
            };
            let mut cmd: Command = (&task).try_into()?;
            let output = String::from_utf8(cmd.output().await?.stdout)?;
            let result: Sentinel = serde_json::from_str(&output)?;
            assert_eq!(result.args.as_slice(), &[bin_path.as_ref(), "hello", "world"]);
            assert_eq!(result.cwd, tempdir.path().join("work").to_str().expect("valid utf-8").to_string());
            Ok(())
        }
    }

}
