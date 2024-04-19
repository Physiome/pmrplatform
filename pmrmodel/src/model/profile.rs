use pmrcore::{
    error::ValueError,
    profile::{
        Profile,
        ViewTaskTemplate,
    },
};

use crate::{
    model::task_template::UserArgRefs,
    registry::ChoiceRegistryCache,
};

#[derive(Debug, serde::Serialize)]
pub struct UserPromptSetRef<'a> {
    // references underlying ViewTaskTemplate.id
    id: i64,
    description: &'a str,
    // the UserArgRefs contain the prompts
    user_arg_refs: UserArgRefs<'a>,
}

#[derive(Debug, serde::Serialize)]
pub struct UserViewProfileRef<'a> {
    // references underlying Profile.id
    id: i64,
    title: &'a str,
    description: &'a str,
    user_prompt_sets: Vec<UserPromptSetRef<'a>>,
}

impl<'a, T> From<(
    &'a ViewTaskTemplate,
    &'a ChoiceRegistryCache<'a, T>,
)> for UserPromptSetRef<'a> {
    fn from((view_task_template, choice_registry): (&'a ViewTaskTemplate, &'a ChoiceRegistryCache<'a, T>)) -> Self {
        Self {
            id: view_task_template.id,
            description: view_task_template.description.as_ref(),
            user_arg_refs: (
                view_task_template.task_template
                    .as_ref()
                    .expect("ViewTaskTemplate.task_template cannot be None here"),
                choice_registry
            ).into(),
        }
    }
}
