use std::ops::{Deref, DerefMut};
use crate::profile::{
    UserPromptGroup,
    UserPromptGroups,
    ViewTaskTemplate,
    ViewTaskTemplates,
};

impl From<Vec<ViewTaskTemplate>> for ViewTaskTemplates {
    fn from(args: Vec<ViewTaskTemplate>) -> Self {
        Self(args)
    }
}

impl From<ViewTaskTemplates> for Vec<ViewTaskTemplate> {
    fn from(value: ViewTaskTemplates) -> Self {
        value.0
    }
}

impl<const N: usize> From<[ViewTaskTemplate; N]> for ViewTaskTemplates {
    fn from(args: [ViewTaskTemplate; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for ViewTaskTemplates {
    type Target = Vec<ViewTaskTemplate>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ViewTaskTemplates {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<UserPromptGroup>> for UserPromptGroups {
    fn from(args: Vec<UserPromptGroup>) -> Self {
        Self(args)
    }
}

impl From<UserPromptGroups> for Vec<UserPromptGroup> {
    fn from(value: UserPromptGroups) -> Self {
        value.0
    }
}

impl<const N: usize> From<[UserPromptGroup; N]> for UserPromptGroups {
    fn from(args: [UserPromptGroup; N]) -> Self {
        Self(args.into())
    }
}

impl Deref for UserPromptGroups {
    type Target = Vec<UserPromptGroup>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UserPromptGroups {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
