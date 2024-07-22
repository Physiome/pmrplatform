use pmrcore::{
    profile::{
        ViewTaskTemplate,
        traits::ViewTaskTemplateBackend,
    },
    task_template::traits::TaskTemplateBackend
};
use crate::{
    error::PlatformError,
    platform::Platform,
};

impl Platform {
    pub async fn adds_view_task_template(
        &self,
        view_task_template: ViewTaskTemplate,
    ) -> Result<i64, PlatformError> {
        let task_template = view_task_template.task_template
            .expect("TaskTemplate must be provided with the ViewTaskTemplate");

        let task_template = TaskTemplateBackend::adds_task_template(
            self.tm_platform.as_ref(),
            task_template,
        ).await?;
        let task_template_id = task_template.id;

        let result = ViewTaskTemplateBackend::insert_view_task_template(
            self.mc_platform.as_ref(),
            &view_task_template.view_key,
            &view_task_template.description,
            task_template_id,
        ).await?;
        Ok(result)
    }

    pub async fn get_view_task_template(
        &self,
        id: i64,
    ) -> Result<ViewTaskTemplate, PlatformError> {
        let mut result = ViewTaskTemplateBackend::select_view_task_template_by_id(
            self.mc_platform.as_ref(),
            id,
        ).await?;

        let id = result.task_template_id;
        let task_template = TaskTemplateBackend::get_task_template_by_id(
            self.tm_platform.as_ref(),
            id,
        ).await?;
        result.task_template = Some(task_template);
        Ok(result)
    }
}

#[cfg(test)]
mod testing {
    use test_pmr::ctrl::create_blank_sqlite_platform;

    #[async_std::test]
    async fn test_smoke() -> anyhow::Result<()> {
        let (_, platform) = create_blank_sqlite_platform().await?;
        let view_task_template = serde_json::from_str(r#"
        {
            "view_key": "example_view",
            "description": "This is an example view",
            "task_template": {
                "bin_path": "/usr/local/bin/example",
                "version_id": "1.0.0",
                "args": [
                    {
                        "flag": null,
                        "flag_joined": false,
                        "prompt": "Example prompt",
                        "default": null,
                        "choice_fixed": false,
                        "choice_source": null,
                        "choices": []
                    },
                    {
                        "flag": null,
                        "flag_joined": false,
                        "prompt": "Another example prompt",
                        "default": null,
                        "choice_fixed": false,
                        "choice_source": null,
                        "choices": [
                            {
                                "to_arg": null,
                                "label": "a null value"
                            },
                            {
                                "to_arg": "",
                                "label": "an empty string"
                            }
                        ]
                    }
                ]
            }
        }
        "#)?;
        let id = platform.adds_view_task_template(view_task_template).await?;
        let result = platform.get_view_task_template(id).await?;
        let vt_uts = result.updated_ts;
        let t_cts = result.task_template.clone().unwrap().created_ts;
        let answer = serde_json::from_str(&format!(r#"
        {{
            "id": 1,
            "view_key": "example_view",
            "description": "This is an example view",
            "task_template_id": 1,
            "updated_ts": {vt_uts},
            "task_template": {{
                "id": 1,
                "bin_path": "/usr/local/bin/example",
                "version_id": "1.0.0",
                "final_task_template_arg_id": 2,
                "created_ts": {t_cts},
                "args": [
                    {{
                        "id": 1,
                        "flag": null,
                        "flag_joined": false,
                        "prompt": "Example prompt",
                        "default": null,
                        "choice_fixed": false,
                        "choice_source": null,
                        "task_template_id": 1,
                        "choices": []
                    }},
                    {{
                        "id": 2,
                        "flag": null,
                        "flag_joined": false,
                        "prompt": "Another example prompt",
                        "default": null,
                        "choice_fixed": false,
                        "choice_source": null,
                        "task_template_id": 1,
                        "choices": [
                            {{
                                "id": 1,
                                "to_arg": null,
                                "task_template_arg_id": 2,
                                "label": "a null value"
                            }},
                            {{
                                "id": 2,
                                "to_arg": "",
                                "task_template_arg_id": 2,
                                "label": "an empty string"
                            }}
                        ]
                    }}
                ]
            }}
        }}
        "#))?;
        assert_eq!(result, answer);
        Ok(())
    }

}
