use futures::future;
use async_trait::async_trait;
#[cfg(not(test))]
use chrono::Utc;
use pmrmodel_base::task::{
    Task,
    TaskArg,
};
use pmrmodel_base::task_template::{
    TaskTemplate,
    TaskTemplateArg,
    TaskTemplateArgChoice,
};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
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

// TODO newtypes for public API for various unsafe user provided data.
// TODO maybe consider something more compact than Vec<String> for return type
// TODO handle arg.join_flag
// TODO handle internal choices (e.g. None value override from choice)
// TODO handle external choices (additional argument?)
fn build_arg(
    value: Option<&str>,
    arg: &TaskTemplateArg,
) -> Result<Vec<String>, ArgumentError> {
    if arg.choice_source.is_some() {
        match (
            arg.prompt.is_some(),
            &arg.flag,
            value,
        ) {
            (false, _, Some(_)) => Err(ArgumentError::UnexpectedValue),
            (_, None, None) =>
                Ok([].into()),
            (true, None, Some(value)) =>
                Ok([value.into()].into()),
            (_, Some(flag), None) =>
                Ok([flag.into()].into()),
            (true, Some(flag), Some(value)) =>
                Ok([flag.into(), value.into()].into()),
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
                Ok([].into()),
            (_, None, Some(default), None) =>
                Ok([default.into()].into()),
            (false, Some(flag), None, None) =>
                Ok([flag.into()].into()),
            (_, Some(flag), Some(default), None) =>
                Ok([flag.into(), default.into()].into()),

            // XXX empty value string supplied by user not handled
            (true, _, None, None) => Err(ArgumentError::ValueExpected),
            (true, None, _, Some(value)) =>
                Ok([value.into()].into()),
            (true, Some(flag), _, Some(value)) =>
                Ok([flag.into(), value.into()].into()),
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
fn test_build_arg_standard_no_choices() {
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
        build_arg(None, &default),
        Ok([].into()),
    );
    assert_eq!(
        &build_arg(None, &none_none_default).unwrap(),
        &["just a default value"],
    );
    assert_eq!(
        &build_arg(None, &none_flag_none).unwrap(),
        &["--flag"],
    );
    assert_eq!(
        &build_arg(None, &none_flag_default).unwrap(),
        &["--flag", "flagged default value"],
    );

    // unexpected values (from user input)
    assert_eq!(
        build_arg(Some("foo"), &default),
        Err(ArgumentError::UnexpectedValue),
    );
    assert_eq!(
        build_arg(Some("foo"), &none_none_default),
        Err(ArgumentError::UnexpectedValue),
    );
    assert_eq!(
        build_arg(Some("foo"), &none_flag_none),
        Err(ArgumentError::UnexpectedValue),
    );
    assert_eq!(
        build_arg(Some("foo"), &none_flag_default),
        Err(ArgumentError::UnexpectedValue),
    );

    // prompted, no response
    assert_eq!(
        build_arg(None, &prompt_none_none),
        Err(ArgumentError::ValueExpected),
    );
    assert_eq!(
        &build_arg(None, &prompt_none_default).unwrap(),
        &["prompted but have default value"],
    );
    assert_eq!(
        &build_arg(None, &prompt_none_dempty).unwrap(),
        &[""],
    );
    assert_eq!(
        build_arg(None, &prompt_flag_none),
        Err(ArgumentError::ValueExpected),
    );
    assert_eq!(
        &build_arg(None, &prompt_flag_default).unwrap(),
        &["-P", "prompted and flagged default value"],
    );
    assert_eq!(
        &build_arg(None, &prompt_flag_dempty).unwrap(),
        &["-P", ""],
    );

    // prompted with non-empty string response
    assert_eq!(
        &build_arg(Some("user value"), &prompt_none_none).unwrap(),
        &["user value"],
    );
    assert_eq!(
        &build_arg(Some("user value"), &prompt_none_default).unwrap(),
        &["user value"],
    );
    assert_eq!(
        &build_arg(Some("user value"), &prompt_none_dempty).unwrap(),
        &["user value"],
    );
    assert_eq!(
        &build_arg(Some("user value"), &prompt_flag_none).unwrap(),
        &["-P", "user value"],
    );
    assert_eq!(
        &build_arg(Some("user value"), &prompt_flag_default).unwrap(),
        &["-P", "user value"],
    );
    assert_eq!(
        &build_arg(Some("user value"), &prompt_flag_dempty).unwrap(),
        &["-P", "user value"],
    );

    // prompted with non-empty string response
    assert_eq!(
        build_arg(Some(""), &prompt_none_none),
        Err(ArgumentError::ValueExpected),
    );
    assert_eq!(
        &build_arg(Some(""), &prompt_none_default).unwrap(),
        &["prompted but have default value"],
    );
    assert_eq!(
        &build_arg(Some(""), &prompt_none_dempty).unwrap(),
        &[""],
    );
    assert_eq!(
        build_arg(Some(""), &prompt_flag_none),
        Err(ArgumentError::ValueExpected),
    );
    assert_eq!(
        &build_arg(Some(""), &prompt_flag_default).unwrap(),
        &["-P", "prompted and flagged default value"],
    );
    assert_eq!(
        &build_arg(Some(""), &prompt_flag_dempty).unwrap(),
        &["-P", ""],
    );

}

#[test]
fn test_build_arg_standard_choices() {
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
        build_arg(None, &none_none),
        Ok([].into()),
    );
    assert_eq!(
        build_arg(None, &none_flag).unwrap(),
        &["--flag"],
    );
    assert_eq!(
        build_arg(None, &prompt_none),
        Ok([].into()),
    );
    assert_eq!(
        build_arg(None, &prompt_flag).unwrap(),
        &["-P"],
    );

}

fn value_to_arg<'a>(
    value: Option<&'a str>,
    arg: &'a TaskTemplateArg,
    choices: &HashMap<&str, Option<&'a str>>,
) -> Result<Option<&'a str>, ArgumentError> {
    let value = match value {
        Some(value) => value,
        None => match &arg.default {
            Some(value) => value,
            None => return Err(ArgumentError::InvalidChoice),
        }
    };
    match choices.get(value) {
        Some(to_arg) => Ok(*to_arg),
        None => match arg.choice_fixed {
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
    let choices = prompt_choices.choices.as_ref().unwrap().build_lookup();
    assert_eq!(
        Ok(None),
        value_to_arg(Some("omit"), &prompt_choices, &choices),
    );
    assert_eq!(
        Ok(Some("")),
        value_to_arg(Some("empty string"), &prompt_choices, &choices),
    );
    assert_eq!(
        Err(ArgumentError::InvalidChoice),
        value_to_arg(Some("invalid choice"), &prompt_choices, &choices),
    );
    assert_eq!(
        Err(ArgumentError::InvalidChoice),
        value_to_arg(None, &prompt_choices, &choices),
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
    let choices = HashMap::from([
        ("default value", Some("the hidden default")),
    ]);
    assert_eq!(
        Ok(Some("the hidden default")),
        value_to_arg(None, &prompt_choices, &choices),
    );
    assert_eq!(
        Ok(Some("the hidden default")),
        value_to_arg(Some("default value"), &prompt_choices, &choices),
    );
    assert_eq!(
        Ok(Some("unmodified value")),
        value_to_arg(Some("unmodified value"), &prompt_choices, &choices),
    );
}
