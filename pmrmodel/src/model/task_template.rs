use futures::future;
use async_trait::async_trait;
#[cfg(not(test))]
use chrono::Utc;
use pmrmodel_base::task::TaskArg;
use pmrmodel_base::task_template::{
    MapToArgRef,
    TaskTemplate,
    TaskTemplateArg,
    TaskTemplateArgChoice,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum ArgumentError {
    UnexpectedValue,
    ValueExpected,
    InvalidChoice,
}

impl Display for ArgumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match &self {
            ArgumentError::UnexpectedValue =>
                "unexpected user value provided",
            ArgumentError::ValueExpected =>
                "user provided value expected but missing",
            ArgumentError::InvalidChoice =>
                "value not a valid choice",
        })
    }
}

type ArgChunk<'a> = [Option<&'a str>; 2];

struct TaskArgBuilder<'a> {
    args: ArgChunk<'a>,
    template: &'a TaskTemplateArg,
}

impl<'a> TaskArgBuilder<'a> {
    fn new(
        args: ArgChunk<'a>,
        template: &'a TaskTemplateArg,
    ) -> Self {
        Self {
            args: args,
            template: template,
        }
    }
}

impl<'a> From<(ArgChunk<'a>, &'a TaskTemplateArg)> for TaskArgBuilder<'a> {
    fn from(item: (ArgChunk<'a>, &'a TaskTemplateArg)) -> Self {
        Self::new(item.0, item.1)
    }
}

impl<'a> Iterator for TaskArgBuilder<'a> {
    type Item = TaskArg;

    fn next(&mut self) -> Option<TaskArg> {
        match (
            self.template.flag_joined,
            self.args[0].is_some(),
            self.args[1].is_some(),
        ) {
            (true, true, true) => Some([
                self.args[0].take().unwrap().into(),
                self.args[1].take().unwrap().into(),
            ].into()),
            (_, true, _) => Some(self.args[0].take().unwrap().into()),
            (_, _, true) => Some(self.args[1].take().unwrap().into()),
            _ => None,
        }
    }
}

#[test]
fn test_arg_builder() {
    let sep = TaskTemplateArg { flag_joined: false, .. Default::default() };
    let join = TaskTemplateArg { flag_joined: true, .. Default::default() };
    let nothing = [None, None];
    let flag = [Some("--flag"), None];
    let value = [None, Some("value")];
    let flag_value = [Some("--flag"), Some("value")];
    let flagvalue = [Some("--flag="), Some("value")];

    assert_eq!(
        TaskArgBuilder::from((flag, &sep))
            .into_iter()
            .collect::<Vec<_>>(),
        vec![
            TaskArg { arg: "--flag".into(), .. Default::default() },
        ]
    );

    assert_eq!(
        TaskArgBuilder::from((value, &sep))
            .into_iter()
            .collect::<Vec<_>>(),
        vec![
            TaskArg { arg: "value".into(), .. Default::default() },
        ]
    );

    assert_eq!(
        TaskArgBuilder::from((flag, &join))
            .into_iter()
            .collect::<Vec<_>>(),
        vec![
            TaskArg { arg: "--flag".into(), .. Default::default() },
        ]
    );

    assert_eq!(
        TaskArgBuilder::from((value, &join))
            .into_iter()
            .collect::<Vec<_>>(),
        vec![
            TaskArg { arg: "value".into(), .. Default::default() },
        ]
    );

    assert_eq!(
        TaskArgBuilder::from((flag_value, &sep))
            .into_iter()
            .collect::<Vec<_>>(),
        vec![
            TaskArg { arg: "--flag".into(), .. Default::default() },
            TaskArg { arg: "value".into(), .. Default::default() },
        ]
    );

    assert_eq!(
        TaskArgBuilder::from((flagvalue, &join))
            .into_iter()
            .collect::<Vec<_>>(),
        vec![
            TaskArg { arg: "--flag=value".into(), .. Default::default() },
        ]
    );

    assert_eq!(
        TaskArgBuilder::from((nothing, &join))
            .into_iter()
            .collect::<Vec<_>>(),
        vec![]
    );

    assert_eq!(
        TaskArgBuilder::from((nothing, &sep))
            .into_iter()
            .collect::<Vec<_>>(),
        vec![]
    );

}

// TODO newtypes for public API for various unsafe user provided data.
// TODO maybe consider something more compact than Vec<String> for return type
// TODO handle arg.join_flag
// TODO handle internal choices (e.g. None value override from choice)
// TODO handle external choices (additional argument?)
fn value_to_argtuple<'a>(
    value: Option<&'a str>,
    arg: &'a TaskTemplateArg,
) -> Result<ArgChunk<'a>, ArgumentError> {
    if arg.choice_source.is_some() {
        match (
            arg.prompt.is_some(),
            &arg.flag,
            value,
        ) {
            (false, _, Some(_)) => Err(ArgumentError::UnexpectedValue),
            (_, None, None) =>
                Ok([None, None]),
            (true, None, Some(value)) =>
                Ok([None, Some(value)]),
            (_, Some(flag), None) =>
                Ok([Some(flag), None]),
            (true, Some(flag), Some(value)) =>
                Ok([Some(flag), Some(value)]),
        }
    }
    else {
        let value = if value == Some("") { None } else { value };
        match (
            arg.prompt.is_some(),
            &arg.flag,
            &arg.default,
            value,
        ) {
            (false, _, _, Some(_)) => Err(ArgumentError::UnexpectedValue),
            (false, None, None, None) =>
                Ok([None, None]),
            (_, None, Some(default), None) =>
                Ok([None, Some(default)]),
            (false, Some(flag), None, None) =>
                Ok([Some(flag), None]),
            (_, Some(flag), Some(default), None) =>
                Ok([Some(flag), Some(default)]),

            // XXX empty value string supplied by user not handled
            (true, _, None, None) => Err(ArgumentError::ValueExpected),
            (true, None, _, Some(value)) =>
                Ok([None, Some(value)]),
            (true, Some(flag), _, Some(value)) =>
                Ok([Some(flag), Some(value)]),
        }
    }
}

/*
TaskTemplateArg

ordered by its id - this means the underlying order cannot be changed, can
only be extended

task_template_id - points to the TaskTemplate.id this arg is associated with
flag - the flag to provide (e.g. `-D`, `--define`)
flag_joined - if false, the value is a separate arg, if true, value is joined wi
th flag, e.g:
                - if flag = `-D`, flag_joined = true, `-Dvalue`
                - if flag = `-D`, flag_joined = false, `-D` `value`
                - if flag = `--define=`, flag_joined = true, `--define=value`
                - if flag = `--define`, flag_joined = false, `--define` `value`
effectively, it should concat the result tuple in matrix at the end.

prompt - if not provided, this will not be prompted to user.
default - if provided, this value be used if user input is an empty string
              - if not provided (i.e. null), and prompt is not null, this must b
e supplied by user

choice_fixed - if true, the provided value for task must be one of the choices
*/


#[test]
fn test_value_to_taskarg_standard_no_choices() {
    let default = TaskTemplateArg { .. Default::default() };
    let none_none_default = TaskTemplateArg {
        default: Some("just a default value".into()),
        .. Default::default()
    };
    let none_flag_none = TaskTemplateArg {
        flag: Some("--flag".into()),
        .. Default::default()
    };
    let none_flag_default = TaskTemplateArg {
        flag: Some("--flag".into()),
        default: Some("flagged default value".into()),
        .. Default::default()
    };
    let prompt_none_none = TaskTemplateArg {
        prompt: Some("Prompt for some user input".into()),
        .. Default::default()
    };
    let prompt_none_default = TaskTemplateArg {
        prompt: Some("Prompt for some user input".into()),
        default: Some("prompted but have default value".into()),
        .. Default::default()
    };
    let prompt_none_dempty = TaskTemplateArg {
        prompt: Some("Prompt for some user input".into()),
        default: Some("".into()),
        .. Default::default()
    };
    let prompt_flag_none = TaskTemplateArg {
        prompt: Some("Prompt for some required user input".into()),
        flag: Some("-P".into()),
        .. Default::default()
    };
    let prompt_flag_default = TaskTemplateArg {
        prompt: Some("Prompt for some optional user input".into()),
        flag: Some("-P".into()),
        default: Some("prompted and flagged default value".into()),
        .. Default::default()
    };
    let prompt_flag_dempty = TaskTemplateArg {
        prompt: Some("Prompt for some optional user input".into()),
        flag: Some("-P".into()),
        default: Some("".into()),
        .. Default::default()
    };

    // default
    assert_eq!(
        value_to_argtuple(None, &default),
        Ok([None, None]),
    );
    assert_eq!(
        value_to_argtuple(None, &none_none_default),
        Ok([None, Some("just a default value")]),
    );
    assert_eq!(
        value_to_argtuple(None, &none_flag_none),
        Ok([Some("--flag"), None]),
    );
    assert_eq!(
        value_to_argtuple(None, &none_flag_default),
        Ok([Some("--flag"), Some("flagged default value")]),
    );

    // unexpected values (from user input)
    assert_eq!(
        value_to_argtuple(Some("foo"), &default),
        Err(ArgumentError::UnexpectedValue),
    );
    assert_eq!(
        value_to_argtuple(Some("foo"), &none_none_default),
        Err(ArgumentError::UnexpectedValue),
    );
    assert_eq!(
        value_to_argtuple(Some("foo"), &none_flag_none),
        Err(ArgumentError::UnexpectedValue),
    );
    assert_eq!(
        value_to_argtuple(Some("foo"), &none_flag_default),
        Err(ArgumentError::UnexpectedValue),
    );

    // prompted, no response
    assert_eq!(
        value_to_argtuple(None, &prompt_none_none),
        Err(ArgumentError::ValueExpected),
    );
    assert_eq!(
        value_to_argtuple(None, &prompt_none_default),
        Ok([None, Some("prompted but have default value")]),
    );
    assert_eq!(
        value_to_argtuple(None, &prompt_none_dempty),
        Ok([None, Some("")]),
    );
    assert_eq!(
        value_to_argtuple(None, &prompt_flag_none),
        Err(ArgumentError::ValueExpected),
    );
    assert_eq!(
        value_to_argtuple(None, &prompt_flag_default),
        Ok([Some("-P"), Some("prompted and flagged default value")]),
    );
    assert_eq!(
        value_to_argtuple(None, &prompt_flag_dempty),
        Ok([Some("-P"), Some("")]),
    );

    // prompted with non-empty string response
    assert_eq!(
        value_to_argtuple(Some("user value"), &prompt_none_none),
        Ok([None, Some("user value")]),
    );
    assert_eq!(
        value_to_argtuple(Some("user value"), &prompt_none_default),
        Ok([None, Some("user value")]),
    );
    assert_eq!(
        value_to_argtuple(Some("user value"), &prompt_none_dempty),
        Ok([None, Some("user value")]),
    );
    assert_eq!(
        value_to_argtuple(Some("user value"), &prompt_flag_none),
        Ok([Some("-P"), Some("user value")]),
    );
    assert_eq!(
        value_to_argtuple(Some("user value"), &prompt_flag_default),
        Ok([Some("-P"), Some("user value")]),
    );
    assert_eq!(
        value_to_argtuple(Some("user value"), &prompt_flag_dempty),
        Ok([Some("-P"), Some("user value")]),
    );

    // prompted with non-empty string response
    assert_eq!(
        value_to_argtuple(Some(""), &prompt_none_none),
        Err(ArgumentError::ValueExpected),
    );
    assert_eq!(
        value_to_argtuple(Some(""), &prompt_none_default),
        Ok([None, Some("prompted but have default value")]),
    );
    assert_eq!(
        value_to_argtuple(Some(""), &prompt_none_dempty),
        Ok([None, Some("")]),
    );
    assert_eq!(
        value_to_argtuple(Some(""), &prompt_flag_none),
        Err(ArgumentError::ValueExpected),
    );
    assert_eq!(
        value_to_argtuple(Some(""), &prompt_flag_default),
        Ok([Some("-P"), Some("prompted and flagged default value")]),
    );
    assert_eq!(
        value_to_argtuple(Some(""), &prompt_flag_dempty),
        Ok([Some("-P"), Some("")]),
    );

}

#[test]
fn test_value_to_taskarg_standard_choices() {
    let none_none = TaskTemplateArg {
        choice_source: Some("".into()),
        .. Default::default()
    };
    let none_flag = TaskTemplateArg {
        flag: Some("--flag".into()),
        choice_source: Some("".into()),
        .. Default::default()
    };
    let prompt_none = TaskTemplateArg {
        prompt: Some("Prompt for some user input".into()),
        choice_source: Some("".into()),
        .. Default::default()
    };
    let prompt_flag = TaskTemplateArg {
        prompt: Some("Prompt for some required user input".into()),
        flag: Some("-P".into()),
        choice_source: Some("".into()),
        .. Default::default()
    };

    assert_eq!(
        value_to_argtuple(None, &none_none),
        Ok([None, None]),
    );
    assert_eq!(
        value_to_argtuple(None, &none_flag),
        Ok([Some("--flag"), None]),
    );
    assert_eq!(
        value_to_argtuple(None, &prompt_none),
        Ok([None, None]),
    );
    assert_eq!(
        value_to_argtuple(None, &prompt_flag),
        Ok([Some("-P"), None]),
    );

}

fn value_from_choices<'a>(
    value: Option<&'a str>,
    arg: &'a TaskTemplateArg,
    choices: Option<&'a MapToArgRef>,
) -> Result<Option<&'a str>, ArgumentError> {
    let value = match value {
        Some(value) => value,
        None => match &arg.default {
            Some(value) => value,
            None => return Err(ArgumentError::InvalidChoice),
        }
    };
    match choices.map(|c| c.get(value)) {
        Some(Some(to_arg)) => Ok(*to_arg),
        None | Some(None) => match arg.choice_fixed {
            true => Err(ArgumentError::InvalidChoice),
            false => Ok(Some(value))
        }
    }
}

#[test]
fn test_validate_choice_value_standard() {
    // to emulate usage of choice within an arg
    let prompt_choices = TaskTemplateArg {
        prompt: Some("Prompt for some user input".into()),
        choices: serde_json::from_str(r#"[
            {
                "to_arg": null,
                "label": "omit"
            },
            {
                "to_arg": "",
                "label": "empty string"
            }
        ]"#).unwrap(),
        choice_fixed: true,
        .. Default::default()
    };
    let choices: MapToArgRef = prompt_choices
        .choices
        .as_ref()
        .unwrap()
        .into();

    assert_eq!(
        Ok(None),
        value_from_choices(
            Some("omit"), &prompt_choices, Some(&choices)),
    );
    assert_eq!(
        Ok(Some("")),
        value_from_choices(
            Some("empty string"), &prompt_choices, Some(&choices)),
    );
    assert_eq!(
        Err(ArgumentError::InvalidChoice),
        value_from_choices(
            Some("invalid choice"), &prompt_choices, Some(&choices)),
    );
    assert_eq!(
        Err(ArgumentError::InvalidChoice),
        value_from_choices(
            None, &prompt_choices, Some(&choices)),
    );
    assert_eq!(
        Err(ArgumentError::InvalidChoice),
        value_from_choices(
            Some("invalid choice"), &prompt_choices, None),
    );
}

#[test]
fn test_validate_choice_value_default() {
    // to emulate usage of choice within an arg
    let prompt_choices = TaskTemplateArg {
        prompt: Some("Prompt for some user input".into()),
        default: Some("default value".into()),
        .. Default::default()
    };
    let choices: MapToArgRef = HashMap::from([
        ("default value", Some("the hidden default")),
    ]).into();
    assert_eq!(
        Ok(Some("the hidden default")),
        value_from_choices(
            None, &prompt_choices, Some(&choices)),
    );
    assert_eq!(
        Ok(Some("the hidden default")),
        value_from_choices(
            Some("default value"), &prompt_choices, Some(&choices)),
    );
    assert_eq!(
        Ok(Some("unmodified value")),
        value_from_choices(
            Some("unmodified value"), &prompt_choices, Some(&choices)),
    );

    assert_eq!(
        Ok(Some("unmodified value")),
        value_from_choices(
            Some("unmodified value"), &prompt_choices, None),
    );
}

#[test]
fn test_validate_choice_values_from_list() {
    // to emulate arg with choice gathered externally for any of these
    // common sequence of string types.
    let prompt_choices = TaskTemplateArg {
        prompt: Some("Prompt for some user input".into()),
        choice_source: Some("file_list".into()),
        .. Default::default()
    };

    let fully_owned_choices: Vec<String> = vec![
        "owned_1".into(),
        "owned_2".into(),
    ];
    assert_eq!(
        Ok(Some("owned_1")),
        value_from_choices(
            Some("owned_1"),
            &prompt_choices,
            Some(&(&fully_owned_choices).into()),
        ),
    );

    let ref_choices: Vec<&str> = vec![
        "str_1",
        "str_2",
    ];
    assert_eq!(
        Ok(Some("str_2")),
        value_from_choices(
            Some("str_2"),
            &prompt_choices,
            Some(&(&ref_choices).into()),
        ),
    );

    let slice = [
        "value_1",
        "value_2",
        "value_3",
    ];
    assert_eq!(
        Ok(Some("value_3")),
        value_from_choices(
            Some("value_3"),
            &prompt_choices,
            Some(&slice.into()),
        ),
    );

}

#[test]
fn test_prototype() {
    let user_input = Some("owned_1");
    let task_template_arg = TaskTemplateArg {
        prompt: Some("Prompt for some user input".into()),
        choice_fixed: true,
        choice_source: Some("file_list".into()),
        .. Default::default()
    };
    let raw_choices: Vec<String> = vec![
        "owned_1".into(),
        "owned_2".into(),
    ];

    let result = (|| { Ok::<Vec<TaskArg>, ArgumentError>(
        TaskArgBuilder::from((
            value_to_argtuple(
                value_from_choices(
                    user_input,
                    &task_template_arg,
                    Some(&MapToArgRef::from(&raw_choices)),
                )?,
                &task_template_arg,
            )?,
            &task_template_arg,
        )).into_iter()
            .collect::<Vec<_>>()
    )})();

    assert_eq!(result, Ok(vec![
        TaskArg { arg: "owned_1".into(), .. Default::default() },
    ]));
}
