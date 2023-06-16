use pmrmodel_base::task::TaskArg;
use pmrmodel_base::task_template::{
    MapToArgRef,
    TaskTemplate,
    TaskTemplateArg,
};
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    collections::HashMap,
    ops::Deref,
};

use crate::registry::ChoiceRegistryCache;
use crate::error::{
    ArgumentError,
    LookupError,
    BuildArgError,
};

type ArgChunk<'a> = [Option<&'a str>; 2];
// the following maps user input by TaskTemplateArg.id
type UserInputMap = HashMap<i64, String>;

#[derive(Debug, PartialEq)]
pub struct TaskArgBuilder<'a> {
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

fn build_arg_chunk<'a, T>(
    user_input: Option<&'a str>,
    task_template_arg: &'a TaskTemplateArg,
    choice_registry_cache: &'a ChoiceRegistryCache<'a, T>,
) -> Result<TaskArgBuilder<'a>, BuildArgError> {
    Ok(TaskArgBuilder::from((
        value_to_argtuple(
            value_from_choices(
                user_input,
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
        build_arg_chunk(item.0, item.1, item.2)
    }
}

// TODO maybe make this part of TaskTemplate's impl?
pub fn task_template_process_user_input<'a, T>(
    user_input: &'a UserInputMap,
    task_template: &'a TaskTemplate,
    choice_registry_cache: &'a ChoiceRegistryCache<'a, T>,
) -> Result<Vec<TaskArgBuilder<'a>>, BuildArgError> {
    Ok(match task_template.args {
        Some(ref args) => args.iter()
            .map(|arg| {
                Ok(build_arg_chunk(
                    user_input.get(&arg.id).map(|x| x.as_str()),
                    &arg,
                    choice_registry_cache,
                )?)
            })
            .collect::<Result<Vec<_>, BuildArgError>>()?,
        None => [].into()
    })
}

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
            (false, _, Some(_)) =>
                Err(ArgumentError::UnexpectedValue(arg.id)),
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

    // unexpected values (from user input)
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

fn value_from_choices<'a>(
    value: Option<&'a str>,
    arg: &'a TaskTemplateArg,
    choices: impl Deref<Target = Option<MapToArgRef<'a>>>,
) -> Result<Option<&'a str>, LookupError> {
    let value = match value {
        Some(value) => value,
        None => match &arg.default {
            Some(value) => value,
            None => return Err(LookupError::TaskTemplateArgNoDefault(arg.id)),
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

    // to emulate lookup of choices from registry cache
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
fn test_validate_choice_value_default() {
    // to emulate usage of choice within an arg
    let prompt_choices = TaskTemplateArg {
        prompt: Some("Prompt for some user input".into()),
        default: Some("default value".into()),
        .. Default::default()
    };
    fn choices() -> Option<MapToArgRef<'static>> {
        Some(HashMap::from([
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

#[cfg(test)]
mod test {
    use pmrmodel_base::task_template::{
        TaskTemplate,
        TaskTemplateArg,
    };
    use pmrmodel_base::task::TaskArg;

    use crate::model::task_template::{
        ArgumentError,
        BuildArgError,
        LookupError,
        TaskArgBuilder,
        UserInputMap,
        task_template_process_user_input,
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
    fn test_process_user_inputs() {
        let user_input = UserInputMap::from([
            (123, "The First Example Model".to_string()),
            (516, "src/README.md".to_string()),
            (894, "src/main/example.model".to_string()),
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
        let processed = task_template_process_user_input(
            &user_input,
            &task_template,
            &cache,
        );
        dbg!(&processed);
        assert!(processed.is_ok());
    }

}
