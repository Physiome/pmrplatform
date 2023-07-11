use async_trait::async_trait;
use pmrmodel_base::task_template::{
    TaskTemplate,
    TaskTemplateArg,
    TaskTemplateArgChoice,
};

#[async_trait]
pub trait TaskTemplateBackend {
    // add a new task template that's open to updates
    async fn add_new_task_template(
        &self,
        bin_path: &str,
        version_id: &str,
    ) -> Result<i64, sqlx::Error>;
    // adds a completed task template.
    async fn add_task_template(
        &self,
        bin_path: &str,
        version_id: &str,
        arguments: &[(
            Option<&str>,
            bool,
            Option<&str>,
            Option<&str>,
            bool,
            Option<&str>,
        )],
    ) -> Result<i64, sqlx::Error>;
    // finalize an open task template.
    async fn finalize_new_task_template(
        &self,
        id: i64,
    ) -> Result<i64, sqlx::Error>;
    async fn add_task_template_arg(
        &self,
        task_template_id: i64,
        flag: Option<&str>,
        flag_joined: bool,
        prompt: Option<&str>,
        default: Option<&str>,
        choice_fixed: bool,
        choice_source: Option<&str>,
    ) -> Result<i64, sqlx::Error>;
    async fn delete_task_template_arg_by_id(
        &self,
        id: i64,
    ) -> Result<Option<TaskTemplateArg>, sqlx::Error>;
    async fn add_task_template_arg_choice(
        &self,
        task_template_arg_id: i64,
        to_arg: Option<&str>,
        label: &str,
    ) -> Result<i64, sqlx::Error>;
    async fn get_task_template_arg_by_id(
        &self,
        id: i64,
    ) -> Result<Option<TaskTemplateArg>, sqlx::Error>;
    async fn delete_task_template_arg_choice_by_id(
        &self,
        id: i64,
    ) -> Result<Option<TaskTemplateArgChoice>, sqlx::Error>;
    async fn get_task_template_by_id(
        &self,
        id: i64,
    ) -> Result<TaskTemplate, sqlx::Error>;
    async fn get_task_template_by_arg_id(
        &self,
        id: i64,
    ) -> Result<TaskTemplate, sqlx::Error>;
}
