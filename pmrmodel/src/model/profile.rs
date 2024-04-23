use pmrcore::profile::{
    ViewTaskTemplate,
    ViewTaskTemplates,
    ViewTaskTemplateProfile,
};
use std::ops::Deref;

use crate::{
    model::task_template::UserArgRefs,
    registry::ChoiceRegistryCache,
};

#[derive(Debug, serde::Serialize)]
pub struct UserPromptGroupRef<'a> {
    // references underlying ViewTaskTemplate.id
    id: i64,
    description: &'a str,
    // the UserArgRefs contain the prompts
    user_args: UserArgRefs<'a>,
}

#[derive(Debug, serde::Serialize)]
pub struct UserViewProfileRef<'a> {
    // references underlying Profile.id
    id: i64,
    title: &'a str,
    description: &'a str,
    user_prompt_groups: UserPromptGroupRefs<'a>,
}

#[derive(Debug, serde::Serialize)]
pub struct UserPromptGroupRefs<'a>(Vec<UserPromptGroupRef<'a>>);

impl<'a> From<Vec<UserPromptGroupRef<'a>>> for UserPromptGroupRefs<'a> {
    fn from(items: Vec<UserPromptGroupRef<'a>>) -> Self {
        Self(items)
    }
}

impl<'a, const N: usize> From<[UserPromptGroupRef<'a>; N]> for UserPromptGroupRefs<'a> {
    fn from(args: [UserPromptGroupRef<'a>; N]) -> Self {
        Self(args.into())
    }
}

impl<'a> Deref for UserPromptGroupRefs<'a> {
    type Target = Vec<UserPromptGroupRef<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> From<(
    &'a ViewTaskTemplate,
    &'a ChoiceRegistryCache<'a, T>,
)> for UserPromptGroupRef<'a> {
    fn from((view_task_template, choice_registry): (&'a ViewTaskTemplate, &'a ChoiceRegistryCache<'a, T>)) -> Self {
        Self {
            id: view_task_template.id,
            description: view_task_template.description.as_ref(),
            user_args: (
                view_task_template.task_template
                    .as_ref()
                    .expect("ViewTaskTemplate.task_template cannot be None here"),
                choice_registry
            ).into(),
        }
    }
}

impl<'a, T> From<(
    &'a ViewTaskTemplates,
    &'a ChoiceRegistryCache<'a, T>,
)> for UserPromptGroupRefs<'a> {
    fn from((view_task_templates, choice_registry): (&'a ViewTaskTemplates, &'a ChoiceRegistryCache<'a, T>)) -> Self {
        view_task_templates.iter()
            .map(|vtt| (vtt, choice_registry).into())
            .collect::<Vec<_>>()
            .into()
    }
}

impl<'a, T> From<(
    &'a ViewTaskTemplateProfile,
    &'a ChoiceRegistryCache<'a, T>,
)> for UserViewProfileRef<'a> {
    fn from((vttp, choice_registry): (&'a ViewTaskTemplateProfile, &'a ChoiceRegistryCache<'a, T>)) -> Self {
        Self {
            id: vttp.profile.id,
            title: vttp.profile.title.as_ref(),
            description: vttp.profile.description.as_ref(),
            user_prompt_groups: vttp.view_task_templates
                .iter()
                .map(|vtt| (vtt, choice_registry).into())
                .collect::<Vec<_>>()
                .into(),
        }
    }
}
