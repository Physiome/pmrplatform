use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Task {
    pub id: i64,
    pub task_template_id: i64,
    pub bin_path: String,
    pub pid: Option<i64>,
    pub created_ts: i64,
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskArgs(Vec<TaskArg>);

mod impls;
pub mod traits;
