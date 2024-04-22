use serde::{Deserialize, Serialize};

use crate::task_template::{
    TaskTemplate,
    UserArgs,
};

// profile module contains miscellaneous structs that are parts that
// may form into a profiles that encapsulate defaults.

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Profile {
    #[serde(default)]
    pub id: i64,
    pub title: String,
    pub description: String,
    pub view_task_templates: Option<ViewTaskTemplates>,
}

// TODO see if the individual structs be better organized if grouped
// into individual modules.

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ViewTaskTemplate {
    #[serde(default)]
    pub id: i64,
    // note that this value, if non-ascii, should be escaped as a view?
    // TODO determine any/if restrictions on range of valid characters.
    pub view_key: String,
    pub description: String,
    #[serde(default)]
    pub task_template_id: i64,
    #[serde(default)]
    pub updated_ts: i64,
    pub task_template: Option<TaskTemplate>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ViewTaskTemplates(Vec<ViewTaskTemplate>);

// The underlying binding record for the relationship
// TODO determine whether this is ultimately necessary
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ProfileView {
    #[serde(default)]
    pub id: i64,
    pub profile_id: i64,
    pub view_task_template_id: i64,
}

// synthesized from the above records from the underlying db; isn't
// typically directly stored in this form in the underlying db.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ViewTaskTemplateProfile {
    pub profile: Profile,
    pub view_task_templates: ViewTaskTemplates,
}

// user facing types for the generation of views, grouped by profiles
// and prompts, for the view to be created.  UserPromptGroup encpsulates
// the UserArgs and the UserViewProfile group the different sets of
// view prompts which will map to a resulting view.

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct UserPromptGroup {
    pub id: i64,
    pub description: String,
    pub user_args: UserArgs,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct UserViewProfile {
    pub id: i64,
    pub title: String,
    pub description: String,
    // TODO convert to plural newtype when available.
    pub user_prompt_groups: Vec<UserPromptGroup>,
}

mod impls;
pub mod traits;
