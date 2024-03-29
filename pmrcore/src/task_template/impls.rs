use std::ops::{Deref, DerefMut};
use crate::task_template::*;

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

impl<'a> Deref for MapToArgRef<'a> {
    type Target = HashMap<&'a str, Option<&'a str>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<HashMap<&'a str, Option<&'a str>>> for MapToArgRef<'a> {
    fn from(value: HashMap<&'a str, Option<&'a str>>) -> Self {
        Self(value)
    }
}

impl<'a> From<&'a HashMap<String, String>> for MapToArgRef<'a> {
    fn from(value: &'a HashMap<String, String>) -> Self {
        value.iter()
            .map(|(key, val)| (key.as_ref(), Some(val.as_ref())))
            .collect::<HashMap<&'_ str, Option<&'_ str>>>()
            .into()
    }
}

impl<'a> From<&'a HashMap<String, Option<String>>> for MapToArgRef<'a> {
    fn from(value: &'a HashMap<String, Option<String>>) -> Self {
        value.iter()
            .map(|(key, val)| (key.as_ref(), val.as_ref().map(|s| s.as_ref())))
            .collect::<HashMap<&'_ str, Option<&'_ str>>>()
            .into()
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

// this extracts the choices that are user facing (displayed for users)
// into a vec.
impl<'a> From<MapToArgRef<'a>> for Vec<&'a str> {
    fn from(value: MapToArgRef<'a>) -> Self {
        let mut result: Self = value.0.into_keys()
            .collect();
        result.sort();
        result
    }
}

impl From<Vec<UserArg>> for UserArgs {
    fn from(args: Vec<UserArg>) -> Self {
        Self(args)
    }
}

impl<const N: usize> From<[UserArg; N]> for UserArgs {
    fn from(args: [UserArg; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for UserArgs {
    type Target = Vec<UserArg>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UserArgs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
