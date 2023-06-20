use std::ops::Deref;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Task {
    pub id: i64,
    pub bin_path: String,
    pub pid: Option<i64>,
    pub create_ts: i64,
    pub start_ts: Option<i64>,
    pub stop_ts: Option<i64>,
    pub exit_status: Option<i64>,
    pub basedir: String,
    pub args: Option<TaskArgs>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct TaskArg {
    pub id: i64,
    pub task_id: i64,
    pub arg: String,
}

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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskArgs(Vec<TaskArg>);

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
