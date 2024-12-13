use itertools::{
    Either,
    Itertools,
};
use pmrcore::{
    profile::ViewTaskTemplate,
    task::{
        Task,
        TaskArg,
    },
    task_template::{
        MapToArgRef,
        TaskTemplate,
        TaskTemplateArg,
        UserArg,
        UserArgs,
        UserChoiceRefs,
        UserInputMap,
    },
};
use std::{
    iter::{
        FlatMap,
        Flatten,
    },
    ops::Deref,
    slice::Iter,
    vec::IntoIter,
};

use crate::registry::ChoiceRegistryCache;
use crate::error::{
    ArgumentError,
    BuildArgError,
    BuildArgErrors,
    LookupError,
};

type ArgChunk<'a> = [Option<&'a str>; 2];

#[derive(Debug, PartialEq)]
pub struct TaskArgBuilder<'a> {
    args: ArgChunk<'a>,
    template: &'a TaskTemplateArg,
}

#[derive(Debug)]
pub struct TaskArgBuilders<'a>(Flatten<IntoIter<TaskArgBuilder<'a>>>);

#[derive(Debug)]
pub struct TaskBuilder<'a>{
    task_template: &'a TaskTemplate,
    arg_builders: TaskArgBuilders<'a>,
}

#[derive(Debug, serde::Serialize)]
pub struct UserArgRef<'a> {
    // this directly references the underlying TaskTemplateArg.id
    id: i64,
    prompt: &'a str,
    default: Option<&'a str>,
    choice_fixed: bool,
    // ideal is to have a single reference to a slice, but for now just
    // have a vec of references, punt dealing with the lifetime of that
    // reference to later when we have a better idea on where the slice
    // actually lives.
    choices: Option<UserChoiceRefs<'a>>,
}

#[derive(Debug, serde::Serialize)]
pub struct UserArgRefs<'a>(Vec<UserArgRef<'a>>);

impl<'a> From<Vec<UserArgRef<'a>>> for UserArgRefs<'a> {
    fn from(args: Vec<UserArgRef<'a>>) -> Self {
        Self(args)
    }
}

impl<'a, const N: usize> From<[UserArgRef<'a>; N]> for UserArgRefs<'a> {
    fn from(args: [UserArgRef<'a>; N]) -> Self {
        Self(args.into())
    }
}

impl<'a> Deref for UserArgRefs<'a> {
    type Target = Vec<UserArgRef<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct UserArgBuilder<'a, I, T> {
    args: I,
    choice_registry_cache: &'a ChoiceRegistryCache<'a, T>,
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

impl<'a> TaskBuilder<'a> {
    fn to_task(self) -> Task {
        Task {
            task_template_id: self.task_template.id,
            bin_path: self.task_template.bin_path.clone(),
            basedir: "".into(),  // TODO, determine what actualy goes there
            args: Some(self.arg_builders.collect::<Vec<_>>().into()),
            .. Default::default()
	}
    }
}

impl<'a, I, T> UserArgBuilder<'a, I, T> {
    fn new(
        args: I,
        choice_registry_cache: &'a ChoiceRegistryCache<'a, T>,
    ) -> Self {
        Self {
            args,
            choice_registry_cache,
        }
    }
}

impl From<TaskBuilder<'_>> for Task {
    fn from(item: TaskBuilder<'_>) -> Self {
        item.to_task()
    }
}

impl<'a> From<(ArgChunk<'a>, &'a TaskTemplateArg)> for TaskArgBuilder<'a> {
    fn from(item: (ArgChunk<'a>, &'a TaskTemplateArg)) -> Self {
        Self::new(item.0, item.1)
    }
}

// TODO need to adapt this for the profile case, where grouping applies
impl<'a, T> From<(
    &'a TaskTemplate,
    &'a ChoiceRegistryCache<'a, T>,
)> for UserArgBuilder<'a, Iter<'a, TaskTemplateArg>, T> {
    fn from(item: (&'a TaskTemplate, &'a ChoiceRegistryCache<'a, T>)) -> Self {
        UserArgBuilder::new(
            (&item.0.args.as_ref())
                .expect("args must have been provided with the template")
                .iter(),
            item.1,
        )
    }
}

impl<'a, T> From<(
    &'a TaskTemplate,
    &'a ChoiceRegistryCache<'a, T>,
)> for UserArgRefs<'a> {
    fn from((task_template, cache): (&'a TaskTemplate, &'a ChoiceRegistryCache<'a, T>)) -> Self {
        UserArgBuilder::from((task_template, cache))
            .collect::<Vec<_>>()
            .into()
    }
}

fn vtt_helper(vtt: &ViewTaskTemplate) -> Iter<TaskTemplateArg> {
    (&vtt.task_template)
        .as_ref()
        .expect("task_template must be provided with view_task_template")
        .args.as_ref()
        .expect("args must have been provided with the template")
        .iter()
}

impl<'a, T> From<(
    &'a [ViewTaskTemplate],
    &'a ChoiceRegistryCache<'a, T>,
)> for UserArgBuilder<
    'a,
    FlatMap<
        Iter<'a, ViewTaskTemplate>,
        Iter<'a, TaskTemplateArg>,
        for<'b> fn(&'b ViewTaskTemplate) -> Iter<'b, TaskTemplateArg>
    >,
    T
> {
    fn from(
        item: (&'a [ViewTaskTemplate], &'a ChoiceRegistryCache<'a, T>)
    ) -> Self {
        Self {
            args: (&item.0)
                .iter()
                .flat_map(vtt_helper)
            ,
            choice_registry_cache: item.1,
        }
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

impl<'a> Iterator for TaskArgBuilders<'a> {
    type Item = TaskArg;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

// TODO Need a grouping by ViewTaskTemplate, which is two levels up?
impl<'a, I: Iterator<Item=&'a TaskTemplateArg>, T> Iterator for UserArgBuilder<'a, I, T> {
    type Item = UserArgRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(arg) = self.args.next() {
            match arg.prompt.as_deref() {
                None => continue,
                Some("") => continue,
                Some(prompt) => {
                    return Some(UserArgRef {
                        id: arg.id,
                        prompt: prompt,
                        default: arg.default.as_deref(),
                        choice_fixed: arg.choice_fixed,
                        choices: self.choice_registry_cache.lookup(&arg)
                            .ok()
                            .as_deref()
                            .map(|r| r
                                .as_ref()
                                .map(|v| v.into())
                                // known empty remains empty
                                .unwrap_or(vec![].into())
                            )
                    })
                }
            }
        }
        None
    }
}

fn arg_build_arg_chunk<'a, T>(
    user_input: Option<&'a str>,
    task_template_arg: &'a TaskTemplateArg,
    choice_registry_cache: &'a ChoiceRegistryCache<'a, T>,
) -> Result<TaskArgBuilder<'a>, BuildArgError> {
    Ok(TaskArgBuilder::from((
        value_to_argtuple(
            value_from_choices(
                value_from_arg_prompt(
                    user_input,
                    &task_template_arg,
                )?,
                &task_template_arg,
                choice_registry_cache.lookup(&task_template_arg)?,
            )?,
            &task_template_arg,
        )?,
        task_template_arg,
    )))
}

type InputArgLookup<'a, T> = (
    Option<&'a str>,
    &'a TaskTemplateArg,
    &'a ChoiceRegistryCache<'a, T>,
);

impl<'a, T> TryFrom<InputArgLookup<'a, T>> for TaskArgBuilder<'a> {
    type Error = BuildArgError;

    fn try_from(
        item: InputArgLookup<'a, T>,
    ) -> Result<TaskArgBuilder<'a>, BuildArgError> {
        arg_build_arg_chunk(item.0, item.1, item.2)
    }
}

impl From<&UserArgRef<'_>> for UserArg {
    fn from(item: &UserArgRef<'_>) -> Self {
        Self {
            id: item.id,
            prompt: item.prompt.to_string(),
            default: item.default.map(|s| s.to_string()),
            choice_fixed: item.choice_fixed,
            choices: item.choices
                .as_ref()
                .map(|choices| choices.into())
        }
    }
}

impl From<&UserArgRefs<'_>> for UserArgs {
    fn from(item: &UserArgRefs<'_>) -> Self {
        item.iter()
            .map(UserArg::from)
            .collect::<Vec<_>>()
            .into()
    }
}

// can't quite do this yet because we didn't fully define how borrowing works, but for now
// the following will do
// impl ToOwned for UserArgRefs<'_> {
impl UserArgRefs<'_> {
    pub fn to_owned(&self) -> UserArgs {
        self.into()
    }
}

impl UserArgRef<'_> {
    pub fn to_owned(&self) -> UserArg {
        self.into()
    }
}

fn task_build_arg_chunk<'a, T>(
    user_input: &'a UserInputMap,
    task_template: &'a TaskTemplate,
    choice_registry_cache: &'a ChoiceRegistryCache<'a, T>,
) -> Result<TaskArgBuilders<'a>, BuildArgErrors> {
    Ok(TaskArgBuilders((match task_template.args {
        Some(ref args) => {
            let (builders, errors): (Vec<_>, Vec<BuildArgError>) = args.iter()
                .map(|arg| {
                    arg_build_arg_chunk(
                        user_input.get(&arg.id).map(|x| x.as_str()),
                        &arg,
                        choice_registry_cache,
                    )
                })
                .partition_map(|r| {
                    match r {
                        Ok(v) => Either::Left(v),
                        Err(v) => Either::Right(v),
                    }
                });
            if errors.len() > 0 {
                Err(BuildArgErrors(errors))
            } else {
                Ok(builders)
            }
        }
        None => Ok([].into())
    })?.into_iter().flatten()))
}

type InputTaskLookup<'a, T> = (
    &'a UserInputMap,
    &'a TaskTemplate,
    &'a ChoiceRegistryCache<'a, T>,
);

impl<'a, T> TryFrom<InputTaskLookup<'a, T>> for TaskArgBuilders<'a> {
    type Error = BuildArgErrors;

    fn try_from(item: InputTaskLookup<'a, T>) -> Result<Self, Self::Error> {
        task_build_arg_chunk(item.0, item.1, item.2)
    }
}

impl<'a, T> TryFrom<InputTaskLookup<'a, T>> for TaskBuilder<'a> {
    type Error = BuildArgErrors;

    fn try_from(item: InputTaskLookup<'a, T>) -> Result<Self, Self::Error> {
        Ok(Self {
            task_template: item.1,
            arg_builders: task_build_arg_chunk(item.0, item.1, item.2)?,
        })
    }
}

fn value_to_argtuple<'a>(
    value: Option<&'a str>,
    arg: &'a TaskTemplateArg,
) -> Result<ArgChunk<'a>, ArgumentError> {
    if arg.choice_source.is_some() {
        match (
            arg.prompt.as_deref(),
            &arg.flag,
            value,
        ) {
            (None, _, Some(_)) =>
                Err(ArgumentError::UnexpectedValue(arg.id)),
            (_, None, None) =>
                Ok([None, None]),
            (Some(""), _, None) =>
                Err(ArgumentError::ValueExpected(arg.id)),
            (Some(_), None, Some(value)) =>
                Ok([None, Some(value)]),
            (_, Some(flag), None) =>
                Ok([Some(flag), None]),
            (Some(_), Some(flag), Some(value)) =>
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
            (false, _, _, Some(_)) =>
                Err(ArgumentError::UnexpectedValue(arg.id)),
            (false, None, None, None) =>
                Ok([None, None]),
            (_, None, Some(default), None) =>
                Ok([None, Some(default)]),
            (false, Some(flag), None, None) =>
                Ok([Some(flag), None]),
            (_, Some(flag), Some(default), None) =>
                Ok([Some(flag), Some(default)]),

            // XXX empty value string supplied by user not handled
            (true, _, None, None) =>
                Err(ArgumentError::ValueExpected(arg.id)),
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

    // unexpected values (can be from user input, or default value NOT
    // being coverted to a None value through its choices).
    assert_eq!(
        value_to_argtuple(Some("foo"), &default),
        Err(ArgumentError::UnexpectedValue(0)),
    );
    assert_eq!(
        value_to_argtuple(Some("foo"), &none_none_default),
        Err(ArgumentError::UnexpectedValue(0)),
    );
    assert_eq!(
        value_to_argtuple(Some("foo"), &none_flag_none),
        Err(ArgumentError::UnexpectedValue(0)),
    );
    assert_eq!(
        value_to_argtuple(Some("foo"), &none_flag_default),
        Err(ArgumentError::UnexpectedValue(0)),
    );

    // prompted, no response
    assert_eq!(
        value_to_argtuple(None, &prompt_none_none),
        Err(ArgumentError::ValueExpected(0)),
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
        Err(ArgumentError::ValueExpected(0)),
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
        Err(ArgumentError::ValueExpected(0)),
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
        Err(ArgumentError::ValueExpected(0)),
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

// TODO this function originally tries to resolve the incoming value to
// the actual value provided by choices, but now the first let actually
// does quite a bit and may benefit a move to another function, but it
// is rather well tested as a complete unit so leave it alone for now?
fn value_from_choices<'a>(
    value: Option<&'a str>,
    arg: &'a TaskTemplateArg,
    choices: impl Deref<Target = Option<MapToArgRef<'a>>>,
) -> Result<Option<&'a str>, LookupError> {
    // this ignores the argument prompt; validation is done elsewhere.
    let value = match value {
        Some(value) => value,
        // if no user value is provided...
        None => match &arg.default {
            // ... use the default if that's provided
            Some(value) => value,
            // ... otherwise, if there are no prompts, shouldn't require
            // a default value.
            None => return match arg.prompt.as_deref() {
                None | Some("") => Ok(None),
                Some(_) => Err(LookupError::TaskTemplateArgNoDefault(arg.id)),
            },
        }
    };
    match choices.as_ref().map(|c| c.get(value)) {
        Some(Some(to_arg)) => Ok(*to_arg),
        None | Some(None) => match arg.choice_fixed {
            true => Err(LookupError::InvalidChoice(arg.id, value.into())),
            false => Ok(Some(value))
        }
    }
}

#[test]
fn test_validate_choice_value_standard() {
    // to emulate usage of choice within an arg
    let arg = TaskTemplateArg {
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

    // to emulate lookup of choices from the argument through the
    // registry cache
    fn choices(arg: &TaskTemplateArg) -> Option<MapToArgRef<'_>> {
        Some(arg
            .choices
            .as_ref()
            .unwrap()
            .into()
        )
    }

    assert_eq!(
        Ok(None),
        value_from_choices(
            Some("omit"), &arg, &choices(&arg)),
    );
    assert_eq!(
        Ok(Some("")),
        value_from_choices(
            Some("empty string"), &arg, &choices(&arg)),
    );
    assert_eq!(
        Err(LookupError::InvalidChoice(0, "invalid choice".into())),
        value_from_choices(
            Some("invalid choice"), &arg, &choices(&arg)),
    );
    assert_eq!(
        Err(LookupError::TaskTemplateArgNoDefault(0)),
        value_from_choices(
            None, &arg, &choices(&arg)),
    );

    assert_eq!(
        Err(LookupError::InvalidChoice(0, "invalid choice".into())),
        value_from_choices(
            Some("invalid choice"), &arg, &None),
    );
}

#[test]
fn test_validate_choice_value_prompt_empty_or_none() {
    fn choices() -> Option<MapToArgRef<'static>> {
        Some(std::collections::HashMap::from([
            ("default value", Some("default value")),
        ]).into())
    }

    let empty_prompt_arg = TaskTemplateArg {
        prompt: Some("".into()),
        .. Default::default()
    };
    assert_eq!(
        Ok(None),
        value_from_choices(
            None, &empty_prompt_arg, &choices()),
    );

    let none_prompt_arg = TaskTemplateArg {
        prompt: None,
        .. Default::default()
    };
    assert_eq!(
        Ok(None),
        value_from_choices(
            None, &none_prompt_arg, &choices()),
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
    fn choices() -> Option<MapToArgRef<'static>> {
        Some(std::collections::HashMap::from([
            ("default value", Some("the hidden default")),
        ]).into())
    }
    assert_eq!(
        Ok(Some("the hidden default")),
        value_from_choices(
            None, &prompt_choices, &choices()),
    );
    assert_eq!(
        Ok(Some("the hidden default")),
        value_from_choices(
            Some("default value"), &prompt_choices, &choices()),
    );
    assert_eq!(
        Ok(Some("unmodified value")),
        value_from_choices(
            Some("unmodified value"), &prompt_choices, &choices()),
    );

    assert_eq!(
        Ok(Some("unmodified value")),
        value_from_choices(
            Some("unmodified value"), &prompt_choices, &None),
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
            &Some((&fully_owned_choices).into()),
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
            &Some((&ref_choices).into()),
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
            &Some(slice.into()),
        ),
    );

}

#[test]
fn test_validate_choice_value_no_prompt_default() {
    // to emulate usage of choice within an arg
    let safe_arg = TaskTemplateArg {
        default: Some("empty string".into()),
        choices: serde_json::from_str(r#"[
            {
                "to_arg": null,
                "label": "empty string"
            }
        ]"#).unwrap(),
        choice_source: Some("".into()),
        choice_fixed: true,
        .. Default::default()
    };

    assert_eq!(
        Ok(None),
        value_from_choices(
            None, &safe_arg, &choices(&safe_arg)),
    );
    assert_eq!(
        Ok(None),
        value_from_choices(
            Some("empty string"), &safe_arg, &choices(&safe_arg)),
    );
    assert_eq!(
        Err(LookupError::InvalidChoice(0, "invalid choice".into())),
        value_from_choices(
            Some("invalid choice"), &safe_arg, &choices(&safe_arg)),
    );

    // this arg _will_ return a value that will cause an eventual
    // lookup error
    let unsafe_arg = TaskTemplateArg {
        default: Some("empty string".into()),
        choices: serde_json::from_str(r#"[
            {
                "to_arg": "non-empty value",
                "label": "empty string"
            }
        ]"#).unwrap(),
        choice_source: Some("".into()),
        choice_fixed: true,
        .. Default::default()
    };

    // to emulate lookup of choices from registry cache; this will
    // return some value and cause an unexpected input failure.
    fn choices(arg: &TaskTemplateArg) -> Option<MapToArgRef<'_>> {
        Some(arg
            .choices
            .as_ref()
            .unwrap()
            .into()
        )
    }

    assert_eq!(
        Ok("non-empty value".into()),
        value_from_choices(
            None, &unsafe_arg, &choices(&unsafe_arg)),
    );
    assert_eq!(
        Ok("non-empty value".into()),
        value_from_choices(
            Some("empty string"), &unsafe_arg, &choices(&unsafe_arg)),
    );
    assert_eq!(
        Err(LookupError::InvalidChoice(0, "invalid choice".into())),
        value_from_choices(
            Some("invalid choice"), &unsafe_arg, &choices(&unsafe_arg)),
    );
}

#[test]
fn test_choice_without_prompt() {
    let prompt_choices = TaskTemplateArg {
        prompt: None,
        choice_source: Some("choices".into()),
        .. Default::default()
    };

    let choices: Vec<String> = vec![
        "choice 1".into(),
        "choice 2".into(),
    ];
    assert_eq!(
        Ok(None),
        value_from_choices(
            None,
            &prompt_choices,
            &Some((&choices).into()),
        ),
    );
    // This will then flow onto value_to_argtuple and result in an
    // unexpected value error.
    assert_eq!(
        Ok(Some("choice 1")),
        value_from_choices(
            Some("choice 1"),
            &prompt_choices,
            &Some((&choices).into()),
        ),
    );
}

#[test]
fn test_choice_prompt_empty_string_no_default() -> anyhow::Result<()> {
    let prompt_choices = TaskTemplateArg {
        prompt: Some("".into()),
        choice_source: Some("choices".into()),
        .. Default::default()
    };

    let good_choices: Vec<String> = vec![
        "default".into(),
    ];
    // this case shouldn't normally happen because no prompt should be
    // generated and thus no way for the user to normally provide this
    assert_eq!(
        Ok(Some("default")),
        value_from_choices(
            Some("default"),
            &prompt_choices,
            &Some((&good_choices).into()),
        ),
    );

    // This will then flow onto value_to_argtuple and result in an
    // value expected error.
    let bad_choices: pmrcore::task_template::TaskTemplateArgChoices = serde_json::from_str(r#"[{
        "to_arg": null,
        "label": "default"
    }]"#)?;
    assert_eq!(
        Ok(None),
        value_from_choices(
            None,
            &prompt_choices,
            &Some((&bad_choices).into()),
        ),
    );
    Ok(())
}

#[test]
fn test_choice_prompt_empty_string_default() -> anyhow::Result<()> {
    let prompt_choices = TaskTemplateArg {
        prompt: Some("".into()),
        choice_source: Some("choices".into()),
        default: Some("default".into()),
        .. Default::default()
    };

    let good_choices: Vec<String> = vec![
        "default".into(),
    ];
    assert_eq!(
        Ok(Some("default")),
        value_from_choices(
            None,
            &prompt_choices,
            &Some((&good_choices).into()),
        ),
    );

    // This will then flow onto value_to_argtuple and result in an
    // value expected error.
    let bad_choices: pmrcore::task_template::TaskTemplateArgChoices = serde_json::from_str(r#"[{
        "to_arg": null,
        "label": "default"
    }]"#)?;
    assert_eq!(
        Ok(None),
        value_from_choices(
            Some("default"),
            &prompt_choices,
            &Some((&bad_choices).into()),
        ),
    );
    Ok(())
}

// this does initial validation - if prompt is none or empty string, fail
fn value_from_arg_prompt<'a>(
    value: Option<&'a str>,
    arg: &'a TaskTemplateArg,
) -> Result<Option<&'a str>, ArgumentError> {
    match value {
        None => Ok(value),
        Some(_) => match arg.prompt.as_deref() {
            None | Some("") => Err(ArgumentError::UnexpectedValue(arg.id)),
            _ => Ok(value),
        }
    }
}

#[test]
fn test_value_from_arg_prompt_standard() {
    let arg = TaskTemplateArg {
        prompt: Some("prompt".into()),
        .. Default::default()
    };
    let input = Some("arg");
    assert_eq!(
        value_from_arg_prompt(input, &arg),
        Ok(input),
    );
}

#[test]
fn test_value_from_arg_prompt_none() {
    let arg = TaskTemplateArg {
        id: 1234,
        prompt: None,
        .. Default::default()
    };
    let input = Some("arg");
    assert_eq!(
        value_from_arg_prompt(input, &arg),
        Err(ArgumentError::UnexpectedValue(1234)),
    );
}

#[test]
fn test_value_from_arg_prompt_empty() {
    let arg = TaskTemplateArg {
        id: 4321,
        prompt: Some("".into()),
        .. Default::default()
    };
    let input = Some("arg");
    assert_eq!(
        value_from_arg_prompt(input, &arg),
        Err(ArgumentError::UnexpectedValue(4321)),
    );
}

#[cfg(test)]
mod test {
    use pmrcore::task_template::{
        TaskTemplate,
        TaskTemplateArg,
        TaskTemplateArgChoices,
        UserArg,
    };
    use pmrcore::task::{
        Task,
        TaskArg,
    };

    use crate::error::{
        ArgumentError,
        BuildArgError,
        LookupError,
    };
    use crate::model::task_template::{
        TaskArgBuilder,
        TaskArgBuilders,
        TaskBuilder,
        UserInputMap,
        UserArgBuilder,
        UserArgRefs,
    };
    use crate::registry::{
        ChoiceRegistry,
        PreparedChoiceRegistry,
        ChoiceRegistryCache,
    };

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

    #[test]
    fn test_build_arg_external() {
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
        let mut registry = PreparedChoiceRegistry::new();
        registry.register("file_list", raw_choices.into());
        let cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);

        let chunk_iter = TaskArgBuilder::try_from((
            user_input,
            &task_template_arg,
            &cache,
        ));
        let result = chunk_iter.unwrap().into_iter().collect::<Vec<_>>();
        assert_eq!(result, vec![
            TaskArg { arg: "owned_1".into(), .. Default::default() },
        ]);

        let chunk_iter = TaskArgBuilder::try_from((
            Some("owned_2"),
            &task_template_arg,
            &cache,
        ));
        let result = chunk_iter.unwrap().into_iter().collect::<Vec<_>>();
        assert_eq!(result, vec![
            TaskArg { arg: "owned_2".into(), .. Default::default() },
        ]);

    }

    #[test]
    fn test_build_arg_internal() {
        let task_template_arg = TaskTemplateArg {
            id: 1,
            flag: Some("--flag".into()),
            prompt: Some("Prompt for more user input".into()),
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
            choice_source: Some("".into()),
            .. Default::default()
        };
        let registry = PreparedChoiceRegistry::new();
        let cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);
        let chunk_iter = TaskArgBuilder::try_from((
            Some("empty string"),
            &task_template_arg,
            &cache,
        ));
        let result = chunk_iter.unwrap().into_iter().collect::<Vec<_>>();
        assert_eq!(result, vec![
            TaskArg { arg: "--flag".into(), .. Default::default() },
            TaskArg { arg: "".into(), .. Default::default() },
        ]);

    }

    #[test]
    fn test_build_arg_no_choices() {
        let task_template_arg = TaskTemplateArg {
            id: 1,
            flag: Some("--flag".into()),
            prompt: Some("Prompt for more user input".into()),
            .. Default::default()
        };
        let registry = PreparedChoiceRegistry::new();
        let cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);
        let chunk_iter = TaskArgBuilder::try_from((
            Some("some value"),
            &task_template_arg,
            &cache,
        ));
        let result = chunk_iter.unwrap().into_iter().collect::<Vec<_>>();
        assert_eq!(result, vec![
            TaskArg { arg: "--flag".into(), .. Default::default() },
            TaskArg { arg: "some value".into(), .. Default::default() },
        ]);
    }

    #[test]
    fn test_build_arg_flag_only() {
        // This is to show how a static value (e.g. subcommands) can be
        // passed without prompting for the user (not through default
        // without a prompt)
        let task_template_arg = TaskTemplateArg {
            id: 1,
            flag: Some("Flag".into()),
            .. Default::default()
        };
        let registry = PreparedChoiceRegistry::new();
        let cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);
        let chunk_iter = TaskArgBuilder::try_from((
            None,
            &task_template_arg,
            &cache,
        ));
        let result = chunk_iter.unwrap().into_iter().collect::<Vec<_>>();
        assert_eq!(result, vec![
            TaskArg { arg: "Flag".into(), .. Default::default() },
        ]);
    }

    #[test]
    fn test_build_arg_default_mapped_none() {
        // This test shows how a default value, if the choices are
        // provided in a way that the default value is mapped back to
        // None, no unexpected output is produced.
        let task_template_arg = TaskTemplateArg {
            id: 1,
            default: Some("default".into()),
            choices: serde_json::from_str(r#"[
                {
                    "to_arg": null,
                    "label": "default"
                }
            ]"#).unwrap(),
            choice_source: Some("".into()),
            .. Default::default()
        };
        let registry = PreparedChoiceRegistry::new();
        let cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);
        let chunk_iter = TaskArgBuilder::try_from((
            None,
            &task_template_arg,
            &cache,
        ));
        let result = chunk_iter.unwrap().into_iter().collect::<Vec<_>>();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_build_arg_default_mapped_some() {
        // This test shows how a default value, if the choices are
        // provided in a way that the default value is mapped to some
        // value, this results in an error.
        let task_template_arg = TaskTemplateArg {
            id: 1,
            default: Some("default".into()),
            choices: serde_json::from_str(r#"[
                {
                    "to_arg": "some value",
                    "label": "default"
                }
            ]"#).unwrap(),
            choice_source: Some("".into()),
            .. Default::default()
        };
        let registry = PreparedChoiceRegistry::new();
        let cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);
        let chunk_iter = TaskArgBuilder::try_from((
            None,
            &task_template_arg,
            &cache,
        ));
        assert_eq!(
            chunk_iter,
            Err(BuildArgError::ArgumentError(
                ArgumentError::UnexpectedValue(1))),
        );
    }

    #[test]
    fn test_build_arg_failure() {
        let arg_ext_choices = TaskTemplateArg {
            id: 1,
            prompt: Some("Prompt for some user input".into()),
            choice_fixed: true,
            choice_source: Some("no_such_registry".into()),
            .. Default::default()
        };
        let arg_with_choices = TaskTemplateArg {
            id: 2,
            flag: Some("--flag".into()),
            prompt: Some("Prompt for more user input".into()),
            choices: Some([].into()),
            choice_fixed: true,
            choice_source: Some("".into()),
            .. Default::default()
        };
        let registry = PreparedChoiceRegistry::new();
        let cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);

        let chunk_iter = TaskArgBuilder::try_from((
            Some("value"),
            &arg_ext_choices,
            &cache,
        ));
        assert_eq!(
            chunk_iter,
            Err(BuildArgError::LookupError(
                LookupError::RegistryMissing(1, "no_such_registry".into())
            ))
        );

        let chunk_iter = TaskArgBuilder::try_from((
            Some("invalid"),
            &arg_with_choices,
            &cache,
        ));
        assert_eq!(
            chunk_iter,
            Err(BuildArgError::LookupError(
                LookupError::InvalidChoice(2, "invalid".into())
            ))
        );

    }

    #[test]
    fn test_prompt_user_inputs_prompts_none() {
        let task_template = TaskTemplate {
            id: 1,
            bin_path: "/usr/local/bin/example".into(),
            version_id: "1.0.0".into(),
            created_ts: 1234567890,
            final_task_template_arg_id: Some(999),
            superceded_by_id: None,
            args: Some([
                TaskTemplateArg {
                    id: 999,
                    task_template_id: 3,
                    flag: Some("build".into()),
                    .. Default::default()
                },
            ].into()),
        };
        let registry = PreparedChoiceRegistry::new();
        let cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);

        let user_prompts = UserArgBuilder::from((
            &task_template,
            &cache,
        )).collect::<Vec<_>>();
        assert_eq!(user_prompts.len(), 0);
    }

    #[test]
    fn test_prompt_user_inputs_prompts_empty_string() {
        let task_template = TaskTemplate {
            id: 1,
            bin_path: "/usr/local/bin/example".into(),
            version_id: "1.0.0".into(),
            created_ts: 1234567890,
            final_task_template_arg_id: Some(999),
            superceded_by_id: None,
            args: Some([
                TaskTemplateArg {
                    id: 999,
                    task_template_id: 3,
                    flag: Some("build".into()),
                    prompt: Some("".into()),
                    .. Default::default()
                },
            ].into()),
        };
        let registry = PreparedChoiceRegistry::new();
        let cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);

        let user_prompts = UserArgBuilder::from((
            &task_template,
            &cache,
        )).collect::<Vec<_>>();
        assert_eq!(user_prompts.len(), 0);
    }

    #[test]
    fn test_prompt_user_inputs_various() -> anyhow::Result<()> {
        let task_template = TaskTemplate {
            id: 3,
            bin_path: "/usr/local/bin/model-processor".into(),
            version_id: "1.3.2".into(),
            created_ts: 1686715614,
            final_task_template_arg_id: Some(4242),
            superceded_by_id: None,
            args: Some([
                TaskTemplateArg {
                    id: 12,
                    task_template_id: 3,
                    flag: Some("build".into()),
                    .. Default::default()
                },
                TaskTemplateArg {
                    id: 123,
                    task_template_id: 3,
                    flag: Some("--title=".into()),
                    flag_joined: true,
                    prompt: Some("Heading for this publication".into()),
                    default: None,
                    choice_fixed: false,
                    choice_source: Some("".into()),
                    choices: None,
                },
                TaskTemplateArg {
                    id: 516,
                    task_template_id: 3,
                    flag: Some("--docsrc".into()),
                    flag_joined: false,
                    prompt: Some("Documentation file".into()),
                    default: None,
                    choice_fixed: false,
                    choice_source: Some("".into()),
                    choices: None,
                },
                TaskTemplateArg {
                    id: 777,
                    task_template_id: 3,
                    flag: Some("--alternative".into()),
                    .. Default::default()
                },
                TaskTemplateArg {
                    id: 894,
                    task_template_id: 3,
                    flag: None,
                    flag_joined: false,
                    prompt: Some("The model to process".into()),
                    default: None,
                    choice_fixed: true,
                    choice_source: Some("git_repo".into()),
                    choices: None,
                },
                TaskTemplateArg {
                    id: 2424,
                    task_template_id: 3,
                    flag: None,
                    flag_joined: false,
                    prompt: Some("Dry run".into()),
                    default: Some("no".into()),
                    choice_fixed: true,
                    choice_source: Some("".into()),
                    choices: serde_json::from_str(r#"[
                        {
                            "to_arg": "--dry-run",
                            "label": "yes"
                        },
                        {
                            "to_arg": null,
                            "label": "no"
                        }
                    ]"#)?,
                },
                TaskTemplateArg {
                    id: 4242,
                    task_template_id: 3,
                    flag: None,
                    flag_joined: false,
                    prompt: Some("Hidden".into()),
                    default: Some("no".into()),
                    choice_fixed: true,
                    choice_source: Some("".into()),
                    choices: serde_json::from_str(r#"[
                        {
                            "to_arg": "-H",
                            "label": "yes"
                        },
                        {
                            "to_arg": null,
                            "label": "no"
                        }
                    ]"#)?,
                },
            ].into()),
        };

        let files: Vec<String> = vec![
            "src/README.md".into(),
            "src/main/example.model".into(),
        ];
        let mut registry = PreparedChoiceRegistry::new();
        registry.register("git_repo", files.into());
        let cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);

        let user_prompts = UserArgBuilder::from((
            &task_template,
            &cache,
        )).collect::<Vec<_>>();
        assert_eq!(user_prompts.len(), 5);

        let user_arg_refs: UserArgRefs = (&task_template, &cache).into();
        assert_eq!(user_arg_refs.len(), 5);

        let json_str = serde_json::to_string(&user_prompts)?;

        // verify that the fully owned version of UserArg is compatible
        let user_args: Vec<UserArg> = serde_json::from_str(&json_str)?;
        assert_eq!(user_args[2], UserArg {
            id: 894,
            prompt: "The model to process".into(),
            default: None,
            choice_fixed: true,
            choices: Some(vec![
                "src/README.md".to_string(),
                "src/main/example.model".to_string(),
            ].into()),
        });

        let value: serde_json::Value = serde_json::from_str(&json_str)?;
        let result: serde_json::Value = serde_json::from_str(r#"[
            {
                "id": 123,
                "prompt": "Heading for this publication",
                "default": null,
                "choice_fixed": false,
                "choices": []
            },
            {
                "id": 516,
                "prompt": "Documentation file",
                "default": null,
                "choice_fixed": false,
                "choices": []
            },
            {
                "id": 894,
                "prompt": "The model to process",
                "default": null,
                "choice_fixed": true,
                "choices": [
                    ["src/README.md", false],
                    ["src/main/example.model", false]
                ]
            },
            {
                "id": 2424,
                "prompt": "Dry run",
                "default": "no",
                "choice_fixed": true,
                "choices": [
                    ["no", false],
                    ["yes", false]
                ]
            },
            {
                "id": 4242,
                "prompt": "Hidden",
                "default": "no",
                "choice_fixed": true,
                "choices": [
                    ["no", false],
                    ["yes", false]
                ]
            }
        ]"#)?;

        assert_eq!(value, result);

        Ok(())
    }

    #[test]
    fn test_process_user_inputs() {
        let user_input = UserInputMap::from([
            (123, "The First Example Model".to_string()),
            (516, "src/README.md".to_string()),
            (894, "src/main/example.model".to_string()),
            (2424, "no".to_string()),
            (4242, "yes".to_string()),
        ]);
        let task_template = TaskTemplate {
            id: 3,
            bin_path: "/usr/local/bin/model-processor".into(),
            version_id: "1.3.2".into(),
            created_ts: 1686715614,
            final_task_template_arg_id: Some(4242),
            superceded_by_id: None,
            args: Some([
                TaskTemplateArg {
                    id: 12,
                    task_template_id: 3,
                    flag: Some("build".into()),
                    .. Default::default()
                },
                TaskTemplateArg {
                    id: 123,
                    task_template_id: 3,
                    flag: Some("--title=".into()),
                    flag_joined: true,
                    prompt: Some("Heading for this publication".into()),
                    default: None,
                    choice_fixed: false,
                    choice_source: Some("".into()),
                    choices: None,
                },
                TaskTemplateArg {
                    id: 516,
                    task_template_id: 3,
                    flag: Some("--docsrc".into()),
                    flag_joined: false,
                    prompt: Some("Documentation file".into()),
                    default: None,
                    choice_fixed: false,
                    choice_source: Some("".into()),
                    choices: None,
                },
                TaskTemplateArg {
                    id: 894,
                    task_template_id: 3,
                    flag: None,
                    flag_joined: false,
                    prompt: Some("The model to process".into()),
                    default: None,
                    choice_fixed: true,
                    choice_source: Some("git_repo".into()),
                    choices: None,
                },
                TaskTemplateArg {
                    id: 2424,
                    task_template_id: 3,
                    flag: None,
                    flag_joined: false,
                    prompt: Some("Dry run".into()),
                    default: Some("no".into()),
                    choice_fixed: true,
                    choice_source: Some("".into()),
                    choices: serde_json::from_str(r#"[
                        {
                            "to_arg": "--dry-run",
                            "label": "yes"
                        },
                        {
                            "to_arg": null,
                            "label": "no"
                        }
                    ]"#).unwrap(),
                },
                TaskTemplateArg {
                    id: 4242,
                    task_template_id: 3,
                    flag: None,
                    flag_joined: false,
                    prompt: Some("Hidden".into()),
                    default: Some("no".into()),
                    choice_fixed: true,
                    choice_source: Some("".into()),
                    choices: serde_json::from_str(r#"[
                        {
                            "to_arg": "-H",
                            "label": "yes"
                        },
                        {
                            "to_arg": null,
                            "label": "no"
                        }
                    ]"#).unwrap(),
                },
            ].into()),
        };

        let files: Vec<String> = vec![
            "src/README.md".into(),
            "src/main/example.model".into(),
        ];
        let mut registry = PreparedChoiceRegistry::new();
        registry.register("git_repo", files.into());
        let cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);
        let processed = TaskArgBuilders::try_from((
            &user_input,
            &task_template,
            &cache,
        )).unwrap();
        let args: Vec<String> = processed
            .map(|a| a.arg.clone())
            .collect();

        assert_eq!(&args, &[
            "build",
            "--title=The First Example Model",
            "--docsrc",
            "src/README.md",
            "src/main/example.model",
            "-H",
        ]);

        let task = Task::from(TaskBuilder::try_from((
            &user_input,
            &task_template,
            &cache,
        )).unwrap());

        assert_eq!(6, task.args.unwrap().len());
        assert_eq!(task.bin_path, task_template.bin_path);
        assert_eq!(task.task_template_id, task_template.id);
    }

    #[test]
    fn test_process_fail_validation_user_inputs() {
        let user_input = UserInputMap::from([
            (1, "invalid_choice1".to_string()),
            (2, "invalid_choice2".to_string()),
            (3, "valid".to_string()),
        ]);
        let task_template = TaskTemplate {
            id: 3,
            bin_path: "/usr/local/bin/model-processor".into(),
            version_id: "1.3.2".into(),
            created_ts: 1686715614,
            final_task_template_arg_id: Some(4242),
            superceded_by_id: None,
            args: Some([
                TaskTemplateArg {
                    id: 1,
                    task_template_id: 3,
                    flag: None,
                    flag_joined: false,
                    prompt: Some("Some path".into()),
                    default: None,
                    choice_fixed: true,
                    choice_source: Some("git_repo".into()),
                    choices: None,
                },
                TaskTemplateArg {
                    id: 2,
                    task_template_id: 3,
                    flag: None,
                    flag_joined: false,
                    prompt: Some("Some other path".into()),
                    default: None,
                    choice_fixed: true,
                    choice_source: Some("git_repo".into()),
                    choices: None,
                },
                TaskTemplateArg {
                    id: 3,
                    task_template_id: 3,
                    flag: None,
                    flag_joined: false,
                    prompt: Some("Dry run".into()),
                    default: Some("no".into()),
                    choice_fixed: true,
                    choice_source: Some("".into()),
                    choices: serde_json::from_str(r#"[
                        {
                            "to_arg": "--dry-run",
                            "label": "valid"
                        }
                    ]"#).unwrap(),
                },
                TaskTemplateArg {
                    id: 4,
                    task_template_id: 3,
                    flag: None,
                    flag_joined: false,
                    prompt: Some("Required".into()),
                    default: None,
                    choice_fixed: false,
                    choice_source: None,
                    choices: None,
                },
            ].into()),
        };

        let files: Vec<String> = vec![
            "src/README.md".into(),
            "src/main/example.model".into(),
        ];
        let mut registry = PreparedChoiceRegistry::new();
        registry.register("git_repo", files.into());
        let cache = ChoiceRegistryCache::from(
            &registry as &dyn ChoiceRegistry<_>);
        let processed = TaskArgBuilders::try_from((
            &user_input,
            &task_template,
            &cache,
        ));

        assert_eq!(processed.unwrap_err().0.as_slice(), &[
            BuildArgError::LookupError(
                LookupError::InvalidChoice(1, "invalid_choice1".to_string())
            ),
            BuildArgError::LookupError(
                LookupError::InvalidChoice(2, "invalid_choice2".to_string())
            ),
            BuildArgError::LookupError(
                LookupError::TaskTemplateArgNoDefault(4)
            ),
        ]);
    }

    #[test]
    fn test_process_fail_validation_none_prompt_with_choice() -> anyhow::Result<()> {
        // Use case for having a choice source set up for an argument
        // without a prompt (i.e. prompt: None) with a default choice
        // is so that the default can be a label that states what the
        // intended usage is, and that the to_arg in the choice should
        // be a none so that this passes - failure will be some string
        // value as that will also fail to generate a task.
        let user_input = UserInputMap::from([]);
        let task_template = TaskTemplate {
            id: 1,
            bin_path: "/bin/demo".into(),
            version_id: "1.3.2".into(),
            created_ts: 1686715614,
            final_task_template_arg_id: Some(1),
            superceded_by_id: None,
            args: Some([
                TaskTemplateArg {
                    id: 1,
                    task_template_id: 1,
                    flag: Some("-a".into()),
                    flag_joined: false,
                    prompt: None,
                    default: Some("validation".into()),
                    choice_fixed: true,
                    choice_source: Some("validation".into()),
                    choices: None,
                },
            ].into()),
        };

        // demonstrate the successful validation case.
        {
            let validation: TaskTemplateArgChoices = serde_json::from_str(r#"[{
                "to_arg": null,
                "label": "validation"
            }]"#)?;
            let mut registry = PreparedChoiceRegistry::new();
            registry.register("validation", validation.into());

            let cache = ChoiceRegistryCache::from(
                &registry as &dyn ChoiceRegistry<_>);
            let user_prompts = UserArgBuilder::from((
                &task_template,
                &cache,
            )).collect::<Vec<_>>();
            assert_eq!(user_prompts.len(), 0);

            let args: Vec<String> = TaskArgBuilders::try_from((
                    &user_input,
                    &task_template,
                    &cache,
                ))?
                .map(|a| a.arg.clone())
                .collect();
            assert_eq!(&args, &["-a"]);
        }

        // demonstrate the failure validation case - see the choice has
        // a failure message.
        {
            let validation: TaskTemplateArgChoices = serde_json::from_str(r#"[{
                "to_arg": "failed",
                "label": "validation"
            }]"#)?;
            let mut registry = PreparedChoiceRegistry::new();
            registry.register("validation", validation.into());

            let cache = ChoiceRegistryCache::from(
                &registry as &dyn ChoiceRegistry<_>);

            let processed = TaskArgBuilders::try_from((
                &user_input,
                &task_template,
                &cache,
            ));
            assert_eq!(processed.unwrap_err().0.as_slice(), &[
                BuildArgError::ArgumentError(
                    ArgumentError::UnexpectedValue(1)
                ),
            ]);
        }

        // The situation where somehow an argument was supplied as user
        // input - it should be ignored.
        {
            let user_input = UserInputMap::from([
                (1, "invalid".to_string()),
            ]);
            let validation: TaskTemplateArgChoices = serde_json::from_str(r#"[
                {
                    "to_arg": "should_fail",
                    "label": "validation"
                },
                {
                    "to_arg": null,
                    "label": "invalid"
                }
            ]"#)?;
            let mut registry = PreparedChoiceRegistry::new();
            registry.register("validation", validation.into());

            let cache = ChoiceRegistryCache::from(
                &registry as &dyn ChoiceRegistry<_>);

            let processed = TaskArgBuilders::try_from((
                &user_input,
                &task_template,
                &cache,
            ));
            assert_eq!(processed.unwrap_err().0.as_slice(), &[
                BuildArgError::ArgumentError(
                    ArgumentError::UnexpectedValue(1)
                ),
            ]);
        }

        Ok(())
    }

    #[test]
    fn test_process_fail_validation_empty_prompt_with_choice() -> anyhow::Result<()> {
        // This sets up an argument that has a choice source that will
        // provide fixed values.  The prompt being an empty string
        // denotes that it must provide an argument, and the default
        // value should point to a label that provide the target value
        // that will be the argument passed to the program backed by
        // the task to be generated.
        //
        // The empty string prompt will hide it from the end user.
        let user_input = UserInputMap::from([]);
        let task_template = TaskTemplate {
            id: 1,
            bin_path: "/bin/demo".into(),
            version_id: "1.3.2".into(),
            created_ts: 1686715614,
            final_task_template_arg_id: Some(1),
            superceded_by_id: None,
            args: Some([
                TaskTemplateArg {
                    id: 1,
                    task_template_id: 1,
                    flag: Some("-a".into()),
                    flag_joined: false,
                    prompt: Some("".into()),
                    default: Some("validation".into()),
                    choice_fixed: true,
                    choice_source: Some("validation".into()),
                    choices: None,
                },
            ].into()),
        };

        // The successful path where the choice source has added the
        // intended argument.
        {
            let validation: TaskTemplateArgChoices = serde_json::from_str(r#"[{
                "to_arg": "passed",
                "label": "validation"
            }]"#)?;
            let mut registry = PreparedChoiceRegistry::new();
            registry.register("validation", validation.into());

            let cache = ChoiceRegistryCache::from(
                &registry as &dyn ChoiceRegistry<_>);
            let user_prompts = UserArgBuilder::from((
                &task_template,
                &cache,
            )).collect::<Vec<_>>();
            assert_eq!(user_prompts.len(), 0);

            let args: Vec<String> = TaskArgBuilders::try_from((
                    &user_input,
                    &task_template,
                    &cache,
                ))?
                .map(|a| a.arg.clone())
                .collect();
            assert_eq!(&args, &["-a", "passed"]);
        }

        // The failure path where the choice source cannot resolve the
        // intended argument for the program, triggering a missing error.
        {
            let validation: TaskTemplateArgChoices = serde_json::from_str(r#"[{
                "to_arg": null,
                "label": "validation"
            }]"#)?;
            let mut registry = PreparedChoiceRegistry::new();
            registry.register("validation", validation.into());

            let cache = ChoiceRegistryCache::from(
                &registry as &dyn ChoiceRegistry<_>);

            let processed = TaskArgBuilders::try_from((
                &user_input,
                &task_template,
                &cache,
            ));
            assert_eq!(processed.unwrap_err().0.as_slice(), &[
                BuildArgError::ArgumentError(
                    ArgumentError::ValueExpected(1)
                ),
            ]);
        }

        // The situation where somehow an argument was supplied as user
        // input - it should be ignored.
        {
            let user_input = UserInputMap::from([
                (1, "invalid".to_string()),
            ]);
            let validation: TaskTemplateArgChoices = serde_json::from_str(r#"[
                {
                    "to_arg": "should_pass",
                    "label": "validation"
                },
                {
                    "to_arg": null,
                    "label": "invalid"
                }
            ]"#)?;
            let mut registry = PreparedChoiceRegistry::new();
            registry.register("validation", validation.into());

            let cache = ChoiceRegistryCache::from(
                &registry as &dyn ChoiceRegistry<_>);

            let processed = TaskArgBuilders::try_from((
                &user_input,
                &task_template,
                &cache,
            ));
            assert_eq!(processed.unwrap_err().0.as_slice(), &[
                BuildArgError::ArgumentError(
                    ArgumentError::UnexpectedValue(1)
                ),
            ]);
        }

        Ok(())
    }

}
