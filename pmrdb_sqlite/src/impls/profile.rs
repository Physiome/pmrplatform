mod profile;
mod profile_views;
mod view_task_template;

mod view_task_template_profile {
    use pmrcore::profile::traits::ViewTaskTemplateProfileBackend;
    use crate::SqliteBackend;

    impl ViewTaskTemplateProfileBackend for SqliteBackend {}
}
