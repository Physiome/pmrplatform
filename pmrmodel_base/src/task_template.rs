use serde::{Deserialize, Serialize};
#[cfg(feature = "sqlx")]
use sqlx::{
    sqlite::SqliteRow,
    FromRow,
    Row,
};
use std::collections::HashMap;
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
            match (
                &self.prompt,
                &self.default,
                self.choice_fixed,
                &self.choice_source.as_deref(),
            ) {
                (None, None, _, _) => None,
                (None, Some(default), _, _) =>
                    Some(format!(">?{:?}?<", &default)),
                (Some(prompt), None, false, _) =>
                    Some(format!("<{}>", &prompt)),
                (Some(prompt), Some(default), false, _) =>
                    Some(format!("[<{}>;default={:?}]",
                        &prompt, &default)),
                (Some(prompt), None, true, None) =>
                    Some(format!("<{};choices={{...}}>", &prompt)),
                (Some(prompt), Some(default), true, None) =>
                    Some(format!("[<{}>;default={:?};choices={{...}}]",
                        &prompt, &default)),
                (Some(prompt), None, true, Some("")) =>
                    Some(format!("<{};choices={{...}}>", &prompt)),
                (Some(prompt), None, true, Some(source)) =>
                    Some(format!("<{};choices={{source:'{}'}}>",
                        &prompt, &source)),
                (Some(prompt), Some(default), true, Some("")) =>
                    Some(format!("[<{}>;default={:?};choices={{...}}]",
                        &prompt, &default)),
                (Some(prompt), Some(default), true, Some(source)) =>
                    Some(format!("<<{}>;default={:?};choices={{source:'{}'}}>",
                        &prompt, &default, &source)),
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

impl<const N: usize> From<[TaskTemplateArg; N]> for TaskTemplateArgs {
    fn from(args: [TaskTemplateArg; N]) -> Self {
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

impl<const N: usize> From<[TaskTemplateArgChoice; N]> for TaskTemplateArgChoices {
    fn from(choices: [TaskTemplateArgChoice; N]) -> Self {
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

// What an owned version might have looked like
// pub struct MapToArg(HashMap<String, Option<String>>);
pub struct MapToArgRef<'a>(HashMap<&'a str, Option<&'a str>>);

impl<'a> From<HashMap<&'a str, Option<&'a str>>> for MapToArgRef<'a> {
    fn from(value: HashMap<&'a str, Option<&'a str>>) -> Self {
        Self(value)
    }
}

impl<'a> Deref for MapToArgRef<'a> {
    type Target = HashMap<&'a str, Option<&'a str>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<&'a TaskTemplateArgChoices> for MapToArgRef<'a> {
    fn from(value: &'a TaskTemplateArgChoices) -> Self {
        value.iter()
            .map(|c| (c.label.as_ref(), c.to_arg.as_deref()))
            .collect::<HashMap<&'_ str, Option<&'_ str>>>()
            .into()
    }
}

impl<'a> From<&'a Vec<String>> for MapToArgRef<'a> {
    fn from(value: &'a Vec<String>) -> Self {
        value.iter()
            .map(|s| (s.as_ref(), Some(s.as_ref())))
            .collect::<HashMap<&'_ str, Option<&'_ str>>>()
            .into()
    }
}

impl<'a> From<&Vec<&'a str>> for MapToArgRef<'a> {
    fn from(value: &Vec<&'a str>) -> Self {
        value.iter()
            .map(|s| (*s, Some(*s)))
            .collect::<HashMap<&'_ str, Option<&'_ str>>>()
            .into()
    }
}

impl<'a, const N: usize> From<[&'a str; N]> for MapToArgRef<'a> {
    fn from(value: [&'a str; N]) -> Self {
        value.iter()
            .map(|s| (*s, Some(*s)))
            .collect::<HashMap<&'_ str, Option<&'_ str>>>()
            .into()
    }
}
