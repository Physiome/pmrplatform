use serde::{Deserialize, Serialize};
#[cfg(feature = "sqlx")]
use sqlx::{
    sqlite::SqliteRow,
    FromRow,
    Row,
};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskTemplate {
    // providing this as a default on deserialize, should serialize omit to hide for API?
    #[serde(default)]
    pub id: i64,
    pub bin_path: String,
    pub version_id: String,
    pub created_ts: i64,
    pub final_task_template_arg_id: Option<i64>,
    pub superceded_by_id: Option<i64>,
    pub args: Option<TaskTemplateArgs>,
}

#[cfg(feature = "sqlx")]
impl<'c> FromRow<'c, SqliteRow> for TaskTemplate {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(TaskTemplate {
            id: row.get(0),
            bin_path: row.get(1),
            version_id: row.get(2),
            created_ts: row.get(3),
            final_task_template_arg_id: row.get(4),
            superceded_by_id: row.get(5),
            args: None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskTemplates(Vec<TaskTemplate>);

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct TaskTemplateArg {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub task_template_id: i64,
    pub flag: Option<String>,
    pub flag_joined: bool,
    pub prompt: Option<String>,
    pub default: Option<String>,
    // choice_fixed == false should imply flag_joined == true for security
    pub choice_fixed: bool,
    pub choice_source: Option<String>,
    // TODO may need an enum instead that disambiguates the DB one and
    // the generated ones provided by alternative sources
    pub choices: Option<TaskTemplateArgChoices>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskTemplateArgs(Vec<TaskTemplateArg>);

/*
Choice can be null to _allow_ the above argument to be null to allow a
null argument, likewise for empty-string for the disambiguation.
*/

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskTemplateArgChoice {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub task_template_arg_id: i64,
    // to the underlying argument.
    pub to_arg: Option<String>,
    // the label is what gets picked by the user.
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskTemplateArgChoices(Vec<TaskTemplateArgChoice>);

// What an owned version might have looked like
// pub struct MapToArg(HashMap<String, Option<String>>);
pub struct MapToArgRef<'a>(HashMap<&'a str, Option<&'a str>>);

#[cfg(feature = "display")]
mod display;
mod impls;
pub mod traits;
