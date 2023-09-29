use pmrcore::{
    exposure::{
        task::traits::ExposureTaskBackend,
        traits::{
            Exposure,
            ExposureFile,
        },
    },
    task::{
        Task,
        traits::TaskBackend,
    },
    task_template::UserArg,
    platform::{
        MCPlatform,
        TMPlatform,
    },
    profile::ViewTaskTemplates,
};
use pmrmodel::{
    error::BuildArgError,
    model::task_template::{
        TaskBuilder,
        UserArgBuilder,
        UserInputMap,
    },
    registry::{
        ChoiceRegistry,
        ChoiceRegistryCache,
    },
};
use pmrctrl::{
    platform::Platform,
    registry::make_choice_registry,
};

use test_pmr::ctrl::create_sqlite_platform;

#[async_std::test]
async fn test_platform_create_exposure_list_files() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    assert_eq!(exposure.list_files()?, &["README", "if1"]);
    assert_eq!(exposure.list_exposure_files().await?.len(), 0);

    // this stops "`ex2` dropped here while still borrowed" before the
    // ex2.list_exposure_files() call
    {
        let _file = exposure.create_file("if1").await?;
        // this is cached from above.
        assert_eq!(exposure.list_exposure_files().await?.len(), 0);
    }
    // load a new copy
    {
        let ex2 = platform.get_exposure(exposure.exposure.id()).await?;
        let files = ex2.list_exposure_files().await?;
        assert_eq!(files, &["if1"]);
    }
    Ok(())
}

#[async_std::test]
async fn test_platform_create_exposure_bad_commit() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let exposure = platform.create_exposure(
        1,
        "0000000000000000000000000000000000000000",
    ).await;
    assert!(exposure.is_err());
    Ok(())
}

#[async_std::test]
async fn test_platform_create_exposure_file_missing() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let file = exposure.create_file("no_such_file").await;
    assert!(file.is_err());
    Ok(())
}

#[async_std::test]
async fn test_platform_create_exposure_file_view_task() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
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
                    "prompt": "Pick a file",
                    "default": null,
                    "choice_fixed": true,
                    "choice_source": "files",
                    "choices": []
                }
            ]
        }
    }
    "#)?;
    let template_id = platform.adds_view_task_template(
        view_task_template).await?;

    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    exposure.create_file("if1").await?;

    let exposure = platform.get_exposure(exposure.exposure.id()).await?;
    let vtt = platform.get_view_task_template(template_id).await?;
    assert_eq!((&vtt).task_template
        .as_ref()
        .expect("should have a valid template here")
        .args
        .as_ref()
        .expect("should have valid arguments here")
        .len(),
        2,
    );

    let registry = make_choice_registry(&exposure)?;
    let cache = ChoiceRegistryCache::from(&registry as &dyn ChoiceRegistry<_>);
    let task_template = vtt.task_template
        .as_ref()
        .expect("should have a valid template here");
    let user_arg_refs = UserArgBuilder::from((
        task_template,
        &cache,
    )).collect::<Vec<_>>();
    let up_str = serde_json::to_string(&user_arg_refs)?;

    let user_args: Vec<UserArg> = serde_json::from_str(&up_str)?;

    assert_eq!(user_args, &[
        UserArg {
            id: 1,
            prompt: "Example prompt".into(),
            default: None,
            choice_fixed: false,
            choices: Some([].into()),
        },
        UserArg {
            id: 2,
            prompt: "Pick a file".into(),
            default: None,
            choice_fixed: true,
            choices: Some([
                "README".into(),
                "if1".into(),
            ].into()),
        },
    ]);

    let user_input = UserInputMap::from([
        (1, "Example answer".to_string()),
        (2, "README".to_string()),
    ]);

    let new_task = platform.adds_task(
        Task::from(
            TaskBuilder::try_from((
                &user_input,
                task_template,
                &cache,
            ))?
        )
    ).await?;

    assert_eq!(new_task.id, 1);

    // TODO may need to revisit this particular test when further API
    // refinements are made; for now just grab the task directly from
    // the internal task management platform.
    // let new_task = TaskBackend::gets_task(&platform.tm_platform, 1);

    // TODO actually tying the task back to the exposure file and thus
    // the appropriate view - this test really is a current proof of
    // concept while figuring stuff out.

    Ok(())
}

async fn make_example_view_tasks<'a, M, T>(
    platform: &'a Platform<'a, M, T>
) -> anyhow::Result<Vec<i64>>
where
    M: MCPlatform + Sized + Sync,
    T: TMPlatform + Sized + Sync,
{
    let mut result: Vec<i64> = Vec::new();
    result.push(platform.adds_view_task_template(
        serde_json::from_str(r#"{
            "view_key": "example_view1",
            "description": "",
            "task_template": {
                "bin_path": "/usr/local/bin/example1",
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
                    }
                ]
            }
        }"#)?
    ).await?);
    result.push(platform.adds_view_task_template(
        serde_json::from_str(r#"{
            "view_key": "example_view2",
            "description": "",
            "task_template": {
                "bin_path": "/usr/local/bin/example",
                "version_id": "1.0.0",
                "args": []
            }
        }"#)?
    ).await?);
    result.push(platform.adds_view_task_template(
        serde_json::from_str(r#"{
            "view_key": "example_view3",
            "description": "",
            "task_template": {
                "bin_path": "/usr/local/bin/example3",
                "version_id": "1.0.0",
                "args": [
                    {
                        "flag": "--file1=",
                        "flag_joined": true,
                        "prompt": "Prompt for file",
                        "default": null,
                        "choice_fixed": true,
                        "choice_source": "files",
                        "choices": []
                    }
                ]
            }
        }"#)?
    ).await?);
    result.push(platform.adds_view_task_template(
        serde_json::from_str(r#"{
            "view_key": "example_view4",
            "description": "",
            "task_template": {
                "bin_path": "/usr/local/bin/example3",
                "version_id": "1.0.0",
                "args": [
                    {
                        "flag": "--file2=",
                        "flag_joined": true,
                        "prompt": "Prompt for alternative file",
                        "default": null,
                        "choice_fixed": true,
                        "choice_source": "files",
                        "choices": []
                    }
                ]
            }
        }"#)?
    ).await?);
    Ok(result)
}

#[async_std::test]
async fn test_platform_file_templates_for_exposure_file() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let vtts = make_example_view_tasks(&platform).await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let exposure_file_id = exposure.create_file("if1").await?
        .exposure_file
        .id();

    let vtt = platform.get_file_templates_for_exposure_file(exposure_file_id).await?;
    assert_eq!(vtt.len(), 0);

    ExposureTaskBackend::set_file_templates(
        &platform.mc_platform,
        exposure_file_id,
        [vtts[0]].into_iter(),
    ).await?;
    let vtt = platform.get_file_templates_for_exposure_file(exposure_file_id).await?;
    assert_eq!(vtt.len(), 1);
    assert_eq!(vtt[0]
        .task_template
        .as_ref()
        .expect("task_template defined")
        .args
        .as_ref()
        .expect("task_template.args defined")
        .len(),
        1,
    );
    assert_eq!(vtt[0].view_key, "example_view1");

    ExposureTaskBackend::set_file_templates(
        &platform.mc_platform,
        exposure_file_id,
        [vtts[1], vtts[2]].into_iter(),
    ).await?;
    let vtt = platform.get_file_templates_for_exposure_file(exposure_file_id).await?;
    assert_eq!(vtt.len(), 2);
    assert_eq!(vtt[0].view_key, "example_view2");
    assert_eq!(vtt[0]
        .task_template
        .as_ref()
        .expect("task_template defined")
        .args
        .as_ref()
        .expect("task_template.args defined")
        .len(),
        0,
    );

    let registry = make_choice_registry(&exposure)?;
    let cache = ChoiceRegistryCache::from(&registry as &dyn ChoiceRegistry<_>);
    let user_arg_refs = UserArgBuilder::from((
        vtt.as_slice(),
        &cache,
    )).collect::<Vec<_>>();
    assert_eq!(user_arg_refs.len(), 1);
    let user_args: Vec<UserArg> = serde_json::from_str(
        &serde_json::to_string(&user_arg_refs)?
    )?;
    assert_eq!(user_args[0].prompt, "Prompt for file");
    Ok(())
}

#[async_std::test]
async fn test_platform_file_templates_user_args_usage() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let vtts = make_example_view_tasks(&platform).await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let exposure_file_id = exposure.create_file("if1").await?
        .exposure_file
        .id();

    ExposureTaskBackend::set_file_templates(
        &platform.mc_platform,
        exposure_file_id,
        [vtts[0], vtts[3]].into_iter(),
    ).await?;
    let efvtts = platform.get_file_templates_for_exposure_file(exposure_file_id).await?;
    let registry = make_choice_registry(&exposure)?;
    let cache = ChoiceRegistryCache::from(&registry as &dyn ChoiceRegistry<_>);
    let user_arg_refs = UserArgBuilder::from((
        efvtts.as_slice(),
        &cache,
    )).collect::<Vec<_>>();
    assert_eq!(user_arg_refs.len(), 2);
    let user_args: Vec<UserArg> = serde_json::from_str(
        &serde_json::to_string(&user_arg_refs)?
    )?;
    assert_eq!(user_args[0].id, 1);
    assert_eq!(user_args[0].prompt, "Example prompt");
    assert_eq!(user_args[1].id, 3);
    assert_eq!(user_args[1].prompt, "Prompt for alternative file");
    // TODO test for alternative ID remaps via manual deletes/updates to the
    // underlying linkage between ViewTaskTemplate and TaskTemplate

    let user_input = UserInputMap::from([
        (1, "Example answer".to_string()),
        (3, "README".to_string()),
    ]);

    // TODO implement this for real (as impl on the templates?)
    // TODO this implementation will need error checking, e.g. per field.
    let tasks = efvtts
        .iter()
        .map(|efvtt| Ok(Task::from(TaskBuilder::try_from((
            &user_input,
            efvtt.task_template
                .as_ref()
                .expect("task_template must have been provided"),
            &cache,
        ))?)))
        .collect::<Result<Vec<_>, BuildArgError>>()?;

    // TODO update serde defaults for Task and friends.
    let answers: Vec<Task> = serde_json::from_str(r#"
    [
        {
            "id": 0,
            "task_template_id": 1,
            "bin_path": "/usr/local/bin/example1",
            "pid": null,
            "created_ts": 0,
            "start_ts": null,
            "stop_ts": null,
            "exit_status": null,
            "basedir": "",
            "args": [
                {
                    "id": 0,
                    "task_id": 0,
                    "arg": "Example answer"
                }
            ]
        },
        {
            "id": 0,
            "task_template_id": 4,
            "bin_path": "/usr/local/bin/example3",
            "pid": null,
            "created_ts": 0,
            "start_ts": null,
            "stop_ts": null,
            "exit_status": null,
            "basedir": "",
            "args": [
                {
                    "id": 0,
                    "task_id": 0,
                    "arg": "--file2=README"
                }
            ]
        }
    ]
    "#)?;
    assert_eq!(&answers, &tasks);

    Ok(())
}
