use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row};

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
    pub args: Option<Vec<TaskTemplateArg>>,
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskTemplates {
    pub task_templates: Vec<TaskTemplate>
}

/*
TaskTemplateArg

ordered by its id - this means the underlying order cannot be changed, can only be
extended
task_template_id - points to the TaskTemplate.id this arg is associated with
flag - the flag to provide (e.g. `-D`, `--define`)
flag_joined - if false, the value is a separate arg, if true, value is joined with flag, e.g:
                - if flag = `-D`, flag_joined = true, `-Dvalue`
                - if flag = `-D`, flag_joined = false, `-D` `value`
                - if flag = `--define=`, flag_joined = true, `--define=value`
                - if flag = `--define`, flag_joined = false, `--define` `value`
effectively, it should concat the result tuple in matrix at the end.

prompt - if not provided, this will not be prompted to user.
default_value - if provided, this value be used if user input is an empty string
              - if not provided (i.e. null), and prompt is not null, this must be supplied by user

The following table applies if no choices?
flag prompt default_value value  result tuple
NULL NULL   NULL          NULL   Ok((None, None))
NULL NULL   NULL          ''     Err("user supplied value without prompt")
NULL NULL   NULL          'val'  Err("user supplied value without prompt")
''   NULL   NULL          NULL   Ok(("", None))
''   NULL   NULL          ''     Err("user supplied value without prompt")
''   NULL   NULL          'val'  Err("user supplied value without prompt")
NULL ''     NULL          NULL   Err("user supplied value missing")
NULL ''     NULL          ''     Err("user supplied value missing")
NULL ''     NULL          'val'  Ok((None, "val",))
''   ''     NULL          NULL   Err("user supplied value missing")
''   ''     NULL          ''     Err("user supplied value missing")
''   ''     NULL          'val'  Ok(("", "val",))
NULL NULL   'def'         NULL   Ok((None, "def"))
NULL NULL   'def'         ''     Err("user supplied value without prompt")
NULL NULL   'def'         'val'  Err("user supplied value without prompt")
''   NULL   'def'         NULL   Ok(("", "def"))
''   NULL   'def'         ''     Err("user supplied value without prompt")
''   NULL   'def'         'val'  Err("user supplied value without prompt")
NULL ''     'def'         NULL   Ok((None, "def",))
NULL ''     'def'         ''     Ok((None, "def",))
NULL ''     'def'         'val'  Ok((None, "val",))
''   ''     'def'         NULL   Ok(("", "def",))
''   ''     'def'         ''     Ok(("", "def",))
''   ''     'def'         'val'  Ok(("", "val",))

The result tuple that has the form `Ok(("", None))` does not have a
corresponding outcome that is user toggleable.  The way that a user can
specify that particular form will require choice that has a NULL value.

production of the final arguments will omit all None values

choice_fixed - if true, the provided value for task must be one of the choices
*/

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskTemplateArg {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub task_template_id: i64,
    pub flag: Option<String>,
    pub flag_joined: bool,
    pub prompt: Option<String>,
    pub default_value: Option<String>,
    pub choice_fixed: bool,
    pub choice_source: Option<String>,
    // TODO may need an enum instead that disambiguates the DB one and
    // the generated ones provided by alternative sources
    pub choices: Option<Vec<TaskTemplateArgChoice>>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskTemplateArgs {
    pub task_template_args: Vec<TaskTemplateArg>
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
    pub value: Option<String>,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct TaskTemplateArgChoices {
    pub task_template_arg_choices: Vec<TaskTemplateArgChoices>
}

/*
testing can be done on the task?

each arg does soemthing like

fn validate_task_arg_against_template_arg(
    task_arg: <Task>,
    template_arg: <TaskTemplateArg>,
) -> Result<(), ErrorString>
*/
