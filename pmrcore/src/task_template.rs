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
    #[serde(default)]
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
    // TODO multiple choices; MapToArgRef technically can support that
    // now but it has been nerfed for now.
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

// UserArg is the user facing version of the TemplateArg - it only
// provides fields that are critical to the end-user while hiding the
// other details that are implementation specific for the server.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct UserArg {
    pub id: i64,
    pub prompt: String,
    pub default: Option<String>,
    pub choice_fixed: bool,
    pub choices: Option<UserChoices>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct UserArgs(Vec<UserArg>);

// value: String,
// selected: bool,
// The selected value is derived from the underlying registry, typically
// denotes a value selected by default.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct UserChoice(pub String, pub bool);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct UserChoices(Vec<UserChoice>);

/*
// TODO figure out if putting the ref version here is best - currently,
// the pmrmodel package holds this struct because the details of its
// construction requires the registry and the goal is to encode that so
// no random new constructor will be provided for the reference based
// version.

pub struct UserArgRef<'a> {
    id: i64,
    prompt: &'a str,
    default: Option<&'a str>,
    choice_fixed: bool,
    choices: Option<&'a [&'a str]>,
}
*/

impl AsRef<UserArg> for UserArg {
    fn as_ref(&self) -> &UserArg {
        self
    }
}

/// `UserInputMap` maps from TaskTemplateArg.id to the user specified
/// input value.  Typically this is specific to some exposure file.
pub type UserInputMap = HashMap<i64, String>;

mod map_to_arg;
#[cfg(feature = "display")]
mod display;
mod impls;
pub mod traits;

pub use map_to_arg::{
    UserChoiceRef,
    UserChoiceRefs,
    MapToArgRef,
};
