use serde::{Deserialize, Serialize};

// profile module contains miscellaneous structs that are parts that
// may form into a profiles that encapsulate defaults.

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Profile {
    pub id: i64,
    pub title: String,
    pub description: String,
}

// TODO see if the individual structs be better organized if grouped
// into individual modules.

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ViewTaskTemplate {
    pub id: i64,
    pub view_key: String,
    pub description: String,
    pub task_template_id: i64,
    pub updated_ts: i64,
    // potential future derived fields
    // pub task_template: Option<crate::task_template::TaskTemplate>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ViewTaskTemplates(Vec<ViewTaskTemplate>);

// The underlying binding record for the relationship
// TODO determine whether this is ultimately necessary
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ProfileView {
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

mod impls;
pub mod traits;
