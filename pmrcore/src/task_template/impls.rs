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

impl From<UserArgs> for Vec<UserArg> {
    fn from(value: UserArgs) -> Self {
        value.0
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

impl From<&str> for UserChoice {
    fn from(arg: &str) -> Self {
        Self(arg.to_string(), false)
    }
}

impl From<Vec<UserChoice>> for UserChoices {
    fn from(args: Vec<UserChoice>) -> Self {
        Self(args)
    }
}

impl<const N: usize> From<[UserChoice; N]> for UserChoices {
    fn from(args: [UserChoice; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for UserChoices {
    type Target = Vec<UserChoice>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UserChoices {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<String>> for UserChoices {
    fn from(args: Vec<String>) -> Self {
        Self(
            args.into_iter()
                .map(|s| UserChoice(s, false))
                .collect()
        )
    }
}
