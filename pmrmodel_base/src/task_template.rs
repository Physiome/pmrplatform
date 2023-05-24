use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};
use std::ops::{Deref, DerefMut};
use std::fmt::{Display, Formatter};

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

impl Display for TaskTemplate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\
            TaskTemplate {{ id: {}, version_id: {:?}, ... }}\n\
            {}{}{}\
            ",
            self.id,
            &self.version_id,
            &self.bin_path,
            &(match &self.args {
                Some(args) => format!("{}", args.iter().fold(
                    String::new(), |acc, arg| acc + " " + &arg.to_string())),
                None => "?arguments missing?".to_string(),
            }),
            if self.final_task_template_arg_id.is_some() {
                ""
            }
            else {
                " ?not finalized?"
            },
        )
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

impl Display for TaskTemplateArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (
            &self.flag, self.flag_joined,
            match (&self.prompt, &self.default, self.choice_fixed) {
                (None, None, _) => None,
                (None, Some(default), _) =>
                    Some(format!("{:?}", &default)),
                (Some(prompt), None, false) =>
                    Some(format!("<{}>", &prompt)),
                (Some(prompt), Some(default), false) =>
                    Some(format!("[<{}>;default={:?}]",
                        &prompt, &default)),
                (Some(prompt), None, true) =>
                    Some(format!("<{};choices={{...}}>", &prompt)),
                (Some(prompt), Some(default), true) =>
                    Some(format!("[<{}>;default={:?};choices={{...}}]",
                        &prompt, &default)),
            }
        ) {
            (None, _, None) => write!(f, ""),
            (Some(flag), _, None) => write!(f, "{}", flag),
            (None, _, Some(arg)) => write!(f, "{}", arg),
            (Some(flag), false, Some(arg)) => write!(f, "{} {}", flag, arg),
            (Some(flag), true, Some(arg)) => write!(f, "{}{}", flag, arg),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskTemplateArgs(Vec<TaskTemplateArg>);

impl From<Vec<TaskTemplateArg>> for TaskTemplateArgs {
    fn from(args: Vec<TaskTemplateArg>) -> Self {
        Self(args)
    }
}

impl From<[TaskTemplateArg; 0]> for TaskTemplateArgs {
    fn from(args: [TaskTemplateArg; 0]) -> Self {
        Self(args.into())
    }
}

impl Deref for TaskTemplateArgs {
    type Target = Vec<TaskTemplateArg>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TaskTemplateArgs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/*
Choice can be null to _allow_ the above argument to be null to allow a null argument,
likewise for empty-string for the disambiguation.
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

impl Display for TaskTemplateArgChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} => {}",
            match self.to_arg.as_deref() {
                Some(s) => format!("{:?}", s),
                None => "<OMITTED>".into(),
            },
            &self.label,)
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskTemplateArgChoices(Vec<TaskTemplateArgChoice>);

impl From<Vec<TaskTemplateArgChoice>> for TaskTemplateArgChoices {
    fn from(choices: Vec<TaskTemplateArgChoice>) -> Self {
        Self(choices)
    }
}

impl From<[TaskTemplateArgChoice; 0]> for TaskTemplateArgChoices {
    fn from(choices: [TaskTemplateArgChoice; 0]) -> Self {
        Self(choices.into())
    }
}

impl Deref for TaskTemplateArgChoices {
    type Target = Vec<TaskTemplateArgChoice>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TaskTemplateArgChoices {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
