use async_trait::async_trait;
use crate::{
    error::BackendError,
    task_template::{
        TaskTemplate,
        TaskTemplateArg,
        TaskTemplateArgChoice,
    },
};

#[async_trait]
pub trait TaskTemplateBackend {
    // add a new task template that's open to updates
    async fn add_task_template(
        &self,
        bin_path: &str,
        version_id: &str,
    ) -> Result<(i64, i64), BackendError>;
    // finalize an open task template.
    async fn finalize_new_task_template(
        &self,
        id: i64,
    ) -> Result<Option<i64>, BackendError>;
    async fn add_task_template_arg(
        &self,
        task_template_id: i64,
        flag: Option<&str>,
        flag_joined: bool,
        prompt: Option<&str>,
        default: Option<&str>,
        choice_fixed: bool,
        choice_source: Option<&str>,
    ) -> Result<i64, BackendError>;
    async fn delete_task_template_arg_by_id(
        &self,
        id: i64,
    ) -> Result<Option<TaskTemplateArg>, BackendError>;
    async fn add_task_template_arg_choice(
        &self,
        task_template_arg_id: i64,
        to_arg: Option<&str>,
        label: &str,
    ) -> Result<i64, BackendError>;
    async fn get_task_template_arg_by_id(
        &self,
        id: i64,
    ) -> Result<Option<TaskTemplateArg>, BackendError>;
    async fn delete_task_template_arg_choice_by_id(
        &self,
        id: i64,
    ) -> Result<Option<TaskTemplateArgChoice>, BackendError>;
    async fn get_task_template_by_id(
        &self,
        id: i64,
    ) -> Result<TaskTemplate, BackendError>;
    async fn get_task_template_by_arg_id(
        &self,
        id: i64,
    ) -> Result<TaskTemplate, BackendError>;

    /// This adds a task template item by the Template struct.
    ///
    /// Note that the provided `id` and `created_ts` within the struct
    /// is ignored as the underlying implementation will generate a new
    /// value for those fields.
    async fn adds_task_template(
        &self,
        item: TaskTemplate,
    ) -> Result<TaskTemplate, BackendError> {
        let (id, created_ts) = self.add_task_template(
            &item.bin_path,
            &item.version_id,
        ).await?;
        let mut tasks = item.args
            .map(|args| args.0)
            .unwrap_or([].into())
            .into_iter()
            .map(|arg| self.adds_task_template_arg(id, arg));
        let mut args = Vec::<TaskTemplateArg>::new();
        while let Some(task) = tasks.next() {
            args.push(task.await?);
        }
        let final_arg_id = self.finalize_new_task_template(id).await?;
        Ok(TaskTemplate {
            id,
            bin_path: item.bin_path,
            version_id: item.version_id,
            created_ts: created_ts,
            final_task_template_arg_id: final_arg_id,
            superceded_by_id: None,
            args: Some(args.into()),
        })
    }

    /// This adds a task template argument item.
    ///
    /// Note that its `task_template_id` is ignored, the `parent_id`
    /// must be provided instead, as this is used as part of the
    /// adds_* chain to add a completely new TaskTemplate item.
    async fn adds_task_template_arg(
        &self,
        parent_id: i64,
        item: TaskTemplateArg,
    ) -> Result<TaskTemplateArg, BackendError> {
        let id = self.add_task_template_arg(
            parent_id,
            item.flag.as_deref(),
            item.flag_joined,
            item.prompt.as_deref(),
            item.default.as_deref(),
            item.choice_fixed,
            item.choice_source.as_deref(),
        ).await?;
        let mut tasks = item.choices
            .map(|choices| choices.0)
            .unwrap_or([].into())
            .into_iter()
            .map(|choice| self.adds_task_template_arg_choice(id, choice));
        let mut choices = Vec::<TaskTemplateArgChoice>::new();
        while let Some(task) = tasks.next() {
            choices.push(task.await?);
        }
        Ok(TaskTemplateArg {
            id,
            task_template_id: parent_id,
            flag: item.flag,
            flag_joined: item.flag_joined,
            prompt: item.prompt,
            default: item.default,
            choice_fixed: item.choice_fixed,
            choice_source: item.choice_source,
            choices: Some(choices.into()),
        })
    }

    /// This adds a task template argument choice item.
    ///
    /// Note that its `task_template_arg_id` is ignored, the `parent_id`
    /// must be provided instead, as this is used as part of the
    /// adds_* chain to add a completely new TaskTemplate item.
    async fn adds_task_template_arg_choice(
        &self,
        parent_id: i64,
        item: TaskTemplateArgChoice,
    ) -> Result<TaskTemplateArgChoice, BackendError> {
        // TODO figure out if non-zero task_template_arg_id need a warning
        let id = self.add_task_template_arg_choice(
            parent_id,
            item.to_arg.as_deref(),
            &item.label,
        ).await?;
        Ok(TaskTemplateArgChoice {
            id,
            task_template_arg_id: parent_id,
            to_arg: item.to_arg,
            label: item.label,
        })
    }
}
