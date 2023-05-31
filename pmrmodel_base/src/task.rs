use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Task {
    pub id: i64,
    pub bin_path: String,
    pub pid: Option<i64>,
    pub start_ts: Option<i64>,
    pub stop_ts: Option<i64>,
    pub exit_status: Option<i64>,
    pub basedir: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskArg {
    pub id: i64,
    pub task_id: i64,
    pub arg: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskArgs(Vec<TaskArg>);
