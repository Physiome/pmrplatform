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
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum BuildArgError {
    UnexpectedValue,
    ValueExpected,
}

impl Display for BuildArgError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match &self {
            BuildArgError::UnexpectedValue =>
                "unexpected user value provided",
            BuildArgError::ValueExpected =>
                "user provided value expected but missing",
        })
    }
}

// TODO maybe consider something more compact than Vec<String> for return type
// TODO handle arg.join_flag
// TODO handle internal choices (e.g. None value override from choice)
// TODO handle external choices (additional argument?)
fn build_arg(
    value: Option<&str>,
    arg: &TaskTemplateArg,
) -> Result<Vec<String>, BuildArgError> {
    match (
        &arg.prompt,
        &arg.flag,
        &arg.default_value,
        value,
    ) {
        (None, _, _, Some(_)) => Err(BuildArgError::UnexpectedValue),
        (None, None, None, None) =>
            Ok([].into()),
        (_, None, Some(default_value), None) =>
            Ok([default_value.into()].into()),
        (None, Some(flag), None, None) =>
            Ok([flag.into()].into()),
        (_, Some(flag), Some(default_value), None) =>
            Ok([flag.into(), default_value.into()].into()),

        // XXX empty value string supplied by user not handled
        (Some(_), _, None, None) => Err(BuildArgError::ValueExpected),
        (Some(_), None, _, Some(value)) =>
            Ok([value.into()].into()),
        (Some(_), Some(flag), _, Some(value)) =>
            Ok([flag.into(), value.into()].into()),
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
default_value - if provided, this value be used if user input is an empty string
              - if not provided (i.e. null), and prompt is not null, this must b
e supplied by user

choice_fixed - if true, the provided value for task must be one of the choices
*/


#[test]
fn test_build_arg_standard() {
    let default_arg = TaskTemplateArg { .. Default::default() };
    let none_none_default = TaskTemplateArg {
        default_value: Some("just a default value".into()),
        .. Default::default()
    };
    let none_flag_none = TaskTemplateArg {
        flag: Some("--flag".into()),
        .. Default::default()
    };
    let none_flag_default = TaskTemplateArg {
        flag: Some("--flag".into()),
        default_value: Some("flagged default value".into()),
        .. Default::default()
    };
    let prompt_none_none = TaskTemplateArg {
        prompt: Some("Prompt for some user input".into()),
        .. Default::default()
    };
    let prompt_none_default = TaskTemplateArg {
        prompt: Some("Prompt for some user input".into()),
        default_value: Some("prompted but have default value".into()),
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
        default_value: Some("prompted and flagged default value".into()),
        .. Default::default()
    };

    // default
    assert_eq!(
        build_arg(None, &default_arg),
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
        build_arg(Some("foo"), &default_arg),
        Err(BuildArgError::UnexpectedValue),
    );
    assert_eq!(
        build_arg(Some("foo"), &none_none_default),
        Err(BuildArgError::UnexpectedValue),
    );
    assert_eq!(
        build_arg(Some("foo"), &none_flag_none),
        Err(BuildArgError::UnexpectedValue),
    );
    assert_eq!(
        build_arg(Some("foo"), &none_flag_default),
        Err(BuildArgError::UnexpectedValue),
    );

    // prompted, no response
    assert_eq!(
        build_arg(None, &prompt_none_none),
        Err(BuildArgError::ValueExpected),
    );
    assert_eq!(
        &build_arg(None, &prompt_none_default).unwrap(),
        &["prompted but have default value"],
    );
    assert_eq!(
        build_arg(None, &prompt_flag_none),
        Err(BuildArgError::ValueExpected),
    );
    assert_eq!(
        &build_arg(None, &prompt_flag_default).unwrap(),
        &["-P", "prompted and flagged default value"],
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
        &build_arg(Some("user value"), &prompt_flag_none).unwrap(),
        &["-P", "user value"],
    );
    assert_eq!(
        &build_arg(Some("user value"), &prompt_flag_default).unwrap(),
        &["-P", "user value"],
    );

}
