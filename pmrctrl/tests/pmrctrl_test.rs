use pmrcore::{
    exposure::{
        task::traits::{
            ExposureTaskTemplateBackend,
            ExposureTaskBackend,
        },
        traits::{
            Exposure as _,
            ExposureFile as _,
            ExposureFileView as _,
            ExposureFileViewBackend,
        },
    },
    task::{
        Task,
        traits::TaskBackend,
    },
    task_template::{
        UserChoiceRef,
        UserChoiceRefs,
        UserArg,
        UserInputMap,
    },
    platform::{
        MCPlatform,
        TMPlatform,
    },
    profile::{
        ViewTaskTemplates,
        UserPromptGroup,
        UserViewProfile,
        traits::{
            ProfileBackend,
            ProfileViewsBackend,
        },
    },
};
use pmrmodel::{
    backend::db::SqliteBackend,
    model::{
        profile::UserViewProfileRef,
        task_template::{
            TaskBuilder,
            UserArgBuilder,
        },
    },
    registry::{
        ChoiceRegistry,
        ChoiceRegistryCache,
        PreparedChoiceRegistry,
    },
};
use pmrctrl::platform::Platform;
use std::{
    path::PathBuf,
    fs::read_to_string,
};

use test_binary::build_test_binary_once;
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
        let ex2 = platform.get_exposure(exposure.exposure().id()).await?;
        let files = ex2.list_exposure_files().await?;
        assert_eq!(files, &["if1"]);
    }
    Ok(())
}

#[async_std::test]
async fn test_exposurectrl_ensure_fs() -> anyhow::Result<()> {
    let (reporoot, platform) = create_sqlite_platform().await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let path1 = reporoot.path()
        .join("data/exposure/1/files/README");
    // cheat by creating the underlying dir
    let expected_dir = reporoot.path().join("data/exposure/1/files");
    std::fs::create_dir_all(&expected_dir)?;
    assert_eq!(
        exposure.ensure_fs()?,
        expected_dir,
    );
    assert!(!path1.exists());

    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let path2 = reporoot.path()
        .join("data/exposure/2/files/README");
    // no cheating this time.
    exposure.ensure_fs()?;
    assert_eq!(
        &read_to_string(path2)?,
        "The readme for import1.\n",
    );

    Ok(())
}

#[async_std::test]
async fn test_platform_create_exposure_map_files_fs() -> anyhow::Result<()> {
    let (reporoot, platform) = create_sqlite_platform().await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;

    let filemap = exposure.map_files_fs()?;
    let path1 = reporoot.path()
        .join("data/exposure/1/files/README")
        .display()
        .to_string();
    assert_eq!(
        filemap.get("README").expect("path present"),
        &path1,
    );

    let exposure = platform.create_exposure(
        3,
        "8ae6e9af37c8bd78614545d0ab807348fc46dcab",
    ).await?;

    let filemap = exposure.map_files_fs()?;
    let path2 = reporoot.path()
        .join("data/exposure/2/files/dir1/nested/file_a")
        .display()
        .to_string();
    assert_eq!(
        filemap.get("dir1/nested/file_a").expect("path present"),
        &path2,
    );
    assert_eq!(
        &read_to_string(path2)?,
        "file_a is new",
    );

    Ok(())
}

#[async_std::test]
async fn test_platform_exposure_ctrl_attach_file() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let exposure_file_id = {
        let exposure_file_ctrl = exposure.create_file("if1").await?;
        exposure_file_ctrl.exposure_file().id()
    };

    let ef_ref = platform.mc_platform
        .get_exposure_file(exposure_file_id)
        .await?;

    let efctrl = exposure.ctrl_file(ef_ref)?;
    let pathinfo = efctrl.pathinfo();
    assert_eq!(pathinfo.path(), "if1");

    let efctrl = exposure.ctrl_path("if1").await?;
    let pathinfo = efctrl.pathinfo();
    assert_eq!(pathinfo.path(), "if1");

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

    let exposure = platform.get_exposure(exposure.exposure().id()).await?;
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

    let registry: PreparedChoiceRegistry = (&exposure).try_into()?;
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
    // let new_task = TaskBackend::gets_task(platform.tm_platform.as_ref(), 1);

    // TODO actually tying the task back to the exposure file and thus
    // the appropriate view - this test really is a current proof of
    // concept while figuring stuff out.

    Ok(())
}

async fn make_example_view_task_templates<'p, M, T>(
    platform: &'p Platform<M, T>
) -> anyhow::Result<Vec<i64>>
where
    M: MCPlatform + Sized + Send + Sync,
    T: TMPlatform + Sized + Send + Sync,
{
    use pmrcore::task_template::traits::TaskTemplateBackend;
    // force insertion of a dummy task template that should shift the
    // id for the ExposureFileTaskTemplate vs TaskTemplate.
    let ttb: &dyn TaskTemplateBackend = platform.tm_platform.as_ref();
    ttb.add_task_template("/bin/dummy", "1.0.0").await?;

    let mut result: Vec<i64> = Vec::new();
    result.push(platform.adds_view_task_template(
        serde_json::from_str(r#"{
            "view_key": "example_view1",
            "description": "Example 1",
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
            "description": "Example 2",
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
            "description": "Example 3",
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
                        "choice_source": "files_default",
                        "choices": []
                    }
                ]
            }
        }"#)?
    ).await?);
    result.push(platform.adds_view_task_template(
        serde_json::from_str(r#"{
            "view_key": "example_view4",
            "description": "Example 4",
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
    result.push(platform.adds_view_task_template(
        serde_json::from_str(r#"{
            "view_key": "default_hidden",
            "description": "",
            "task_template": {
                "bin_path": "/usr/local/bin/hidden_example",
                "version_id": "1.0.0",
                "args": [
                    {
                        "flag": "--workspace_file_path=",
                        "flag_joined": true,
                        "prompt": "",
                        "default": "workspace_file_path",
                        "choice_fixed": true,
                        "choice_source": "workspace_file_path",
                        "choices": []
                    },
                    {
                        "flag": "--working_dir=",
                        "flag_joined": true,
                        "prompt": "",
                        "default": "working_dir",
                        "choice_fixed": true,
                        "choice_source": "working_dir",
                        "choices": []
                    }
                ]
            }
        }"#)?
    ).await?);
    Ok(result)
}

async fn make_example_runnable_task_templates<'p, M, T>(
    platform: &'p Platform<M, T>
) -> anyhow::Result<Vec<i64>>
where
    M: MCPlatform + Sized + Send + Sync,
    T: TMPlatform + Sized + Send + Sync,
{
    build_test_binary_once!(sentinel, "../testing");
    build_test_binary_once!(exit_code, "../testing");
    let sentinel = path_to_sentinel().into_string().expect("a valid string");
    let exit_code = path_to_exit_code().into_string().expect("a valid string");

    let mut result: Vec<i64> = Vec::new();
    result.push(platform.adds_view_task_template(
        serde_json::from_str(&format!(r#"{{
            "view_key": "sentinel_0",
            "description": "Sentinel 0",
            "task_template": {{
                "bin_path": "{sentinel}",
                "version_id": "1.0.0",
                "args": []
            }}
        }}"#))?
    ).await?);
    result.push(platform.adds_view_task_template(
        serde_json::from_str(&format!(r#"{{
            "view_key": "exit_code_0",
            "description": "Exit Code 0",
            "task_template": {{
                "bin_path": "{exit_code}",
                "version_id": "1.0.0",
                "args": [
                    {{
                        "flag": null,
                        "flag_joined": false,
                        "prompt": "Exit Code",
                        "default": null,
                        "choice_fixed": false,
                        "choice_source": null,
                        "choices": []
                    }}
                ]
            }}
        }}"#))?
    ).await?);
    Ok(result)
}

#[async_std::test]
async fn test_platform_file_templates_for_exposure_file() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let vtts = make_example_view_task_templates(&platform).await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    // this is now needed to avoid borrow checker getting confused about
    // the order of which vttc is freed (before exposure_file_ctrl, ensure
    // that we drop that.
    let exposure_file_ctrl = exposure.create_file("if1").await?;
    let exposure_file_id = {
        let exposure_file_ref = exposure_file_ctrl.exposure_file();
        exposure_file_ref.id()
    };

    let vttc = exposure_file_ctrl.build_vttc().await?;
    let vtt: &ViewTaskTemplates = (&vttc).into();
    assert_eq!(vtt.len(), 0);

    ExposureTaskTemplateBackend::set_file_templates(
        platform.mc_platform.as_ref(),
        exposure_file_id,
        &[vtts[0]],
    ).await?;
    let vttc = exposure_file_ctrl.build_vttc().await?;
    let vtt: &ViewTaskTemplates = (&vttc).into();
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

    assert_eq!(
        vttc.get_arg(&1).expect("arg").prompt,
        Some("Example prompt".to_string())
    );

    ExposureTaskTemplateBackend::set_file_templates(
        platform.mc_platform.as_ref(),
        exposure_file_id,
        &[vtts[1], vtts[2]],
    ).await?;
    let vttc = exposure_file_ctrl.build_vttc().await?;
    let vtt: &ViewTaskTemplates = (&vttc).into();
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

    let registry: PreparedChoiceRegistry = (&exposure).try_into()?;
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
    let mut exposure_basedir = PathBuf::new();
    exposure_basedir.push(platform.data_root());
    exposure_basedir.push("exposure");
    exposure_basedir.push("1");
    let exposure_file_basedir = exposure_basedir.join("1");
    let exposure_basedir_readme = exposure_basedir.join("files/README");
    let exposure_basedir_readme = exposure_basedir_readme.display();

    let vtts = make_example_view_task_templates(&platform).await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let efc = exposure.create_file("if1").await?;
    let exposure_file_id = efc
        .exposure_file()
        .id();

    ExposureTaskTemplateBackend::set_file_templates(
        platform.mc_platform.as_ref(),
        exposure_file_id,
        &[vtts[0], vtts[3]],
    ).await?;
    assert_eq!(vtts[0], 1);
    assert_eq!(vtts[3], 4);

    let efvttsc = efc.build_vttc().await?;
    let user_arg_refs = efvttsc.create_user_arg_refs().await?;
    let user_args: Vec<UserArg> = user_arg_refs.iter()
        .map(|a| a.into())
        .collect();
    assert_eq!(user_arg_refs.len(), 2);
    assert_eq!(user_args[0].id, 1);
    assert_eq!(user_args[0].prompt, "Example prompt");
    assert_eq!(user_args[1].id, 3);
    assert_eq!(user_args[1].prompt, "Prompt for alternative file");

    let user_prompt_groups = efvttsc.create_user_prompt_groups().await?;
    let upg_str = serde_json::to_string(&user_prompt_groups)?;

    let upg: Vec<UserPromptGroup> = serde_json::from_str(&upg_str)?;
    assert_eq!(upg, &[
        UserPromptGroup {
            id: 1,
            description: "Example 1".into(),
            user_args: [
                UserArg {
                    id: 1,
                    prompt: "Example prompt".into(),
                    default: None,
                    choice_fixed: false,
                    choices: Some([].into()),
                }
            ].into(),
        },
        UserPromptGroup {
            id: 4,
            description: "Example 4".into(),
            user_args: [
                UserArg {
                    id: 3,
                    prompt: "Prompt for alternative file".into(),
                    default: None,
                    choice_fixed: true,
                    choices: Some(["README".into(), "if1".into()].into()),
                }
            ].into(),
        },
    ]);

    // TODO test for alternative ID remaps via manual deletes/updates to the
    // underlying linkage between ViewTaskTemplate and TaskTemplate

    let user_input = UserInputMap::from([
        (1, "Example answer".to_string()),
        (3, "README".to_string()),
    ]);

    let tasks = efvttsc.create_tasks_from_input(&user_input)?
        .into_iter()
        .map(<(i64, Task)>::from)
        .collect::<Vec<_>>();

    let exposure_file_basedir_view1 = exposure_file_basedir
        .join("example_view1")
        .display()
        .to_string();
    let exposure_file_basedir_view4 = exposure_file_basedir
        .join("example_view4")
        .display()
        .to_string();

    let answers: Vec<(i64, Task)> = serde_json::from_str(&format!(r#"
    [
        [1, {{
            "id": 0,
            "task_template_id": 2,
            "bin_path": "/usr/local/bin/example1",
            "pid": null,
            "created_ts": 0,
            "start_ts": null,
            "stop_ts": null,
            "exit_status": null,
            "basedir": "{exposure_file_basedir_view1}",
            "args": [
                {{
                    "id": 0,
                    "task_id": 0,
                    "arg": "Example answer"
                }}
            ]
        }}],
        [4, {{
            "id": 0,
            "task_template_id": 5,
            "bin_path": "/usr/local/bin/example3",
            "pid": null,
            "created_ts": 0,
            "start_ts": null,
            "stop_ts": null,
            "exit_status": null,
            "basedir": "{exposure_file_basedir_view4}",
            "args": [
                {{
                    "id": 0,
                    "task_id": 0,
                    "arg": "--file2={exposure_basedir_readme}"
                }}
            ]
        }}]
    ]
    "#))?;
    assert_eq!(&answers, &tasks);

    // since the one above was consumed for inspection, repeat that call
    // and pass the new one for processing.
    let tasks = efvttsc.create_tasks_from_input(&user_input)?;
    let result = efc.process_vttc_tasks(tasks).await?;
    assert_eq!(result.len(), 2);
    let (efv_id, _) = result[0];

    // TODO finalize the ExposureFileViewTask handling via the platform
    // but for now just use the underlying and find out whether the
    // tasks have been correctly queued.

    let etb: &dyn ExposureTaskBackend = platform.mc_platform.as_ref();
    let et1 = etb.select_task_for_view(efv_id).await?
        .expect("not none");
    assert_eq!(et1.id, 1);
    assert_eq!(et1.exposure_file_view_id, 1);
    assert_eq!(et1.view_task_template_id, 1);
    assert_eq!(et1.ready, false);

    let tb: &dyn TaskBackend = platform.tm_platform.as_ref();
    let task1 = tb.gets_task(et1.task_id.expect("not none")).await?;
    let created_ts = task1.created_ts;
    let answer: Task = serde_json::from_str(&format!(r#"
    {{
        "id": 1,
        "task_template_id": 2,
        "bin_path": "/usr/local/bin/example1",
        "pid": null,
        "created_ts": {created_ts},
        "start_ts": null,
        "stop_ts": null,
        "exit_status": null,
        "basedir": "{exposure_file_basedir_view1}",
        "args": [
            {{
                "id": 1,
                "task_id": 1,
                "arg": "Example answer"
            }}
        ]
    }}
    "#))?;
    assert_eq!(answer, task1);

    Ok(())
}

#[async_std::test]
async fn test_platform_vtt_profile() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let vtts = make_example_view_task_templates(&platform).await?;
    // add another one
    let vtt_final = platform.adds_view_task_template(
        serde_json::from_str(r#"{
            "view_key": "more_args",
            "description": "Just to have more arguments",
            "task_template": {
                "bin_path": "/usr/local/bin/args",
                "version_id": "1.0.0",
                "args": [
                    {
                        "flag": "-A",
                        "flag_joined": false,
                        "prompt": "First question in this group",
                        "default": null,
                        "choice_fixed": false,
                        "choice_source": null,
                        "choices": []
                    },
                    {
                        "flag": "-B",
                        "flag_joined": true,
                        "prompt": "Second question in this group",
                        "default": null,
                        "choice_fixed": false,
                        "choice_source": null,
                        "choices": []
                    },
                    {
                        "flag": "-C",
                        "flag_joined": true,
                        "prompt": "Final multiple choice",
                        "default": null,
                        "choice_fixed": false,
                        "choice_source": "",
                        "choices": [
                            {
                                "to_arg": "Picked A",
                                "label": "Choice A"
                            },
                            {
                                "to_arg": "Picked B",
                                "label": "Choice B"
                            },
                            {
                                "to_arg": "Picked C",
                                "label": "Choice C"
                            }
                        ]
                    }
                ]
            }
        }"#)?
    ).await?;
    let pb: &dyn ProfileBackend = platform.mc_platform.as_ref();
    let id = pb.insert_profile(
        "Profile 1",
        "Just an example profile",
    ).await?;
    let pvb: &dyn ProfileViewsBackend = platform.mc_platform.as_ref();
    pvb.insert_profile_views(id, vtts[1]).await?;
    pvb.insert_profile_views(id, vtts[2]).await?;
    pvb.insert_profile_views(id, vtts[3]).await?;
    pvb.insert_profile_views(id, vtts[4]).await?;
    pvb.insert_profile_views(id, vtt_final).await?;

    let vttp = platform.get_view_task_template_profile(id).await?;
    let registry = PreparedChoiceRegistry::new();
    let cache = ChoiceRegistryCache::from(
        &registry as &dyn ChoiceRegistry<_>);

    let uvpr: UserViewProfileRef = (&vttp, &cache).into();
    // emulate trip from API to end user JSON read
    let uvp: UserViewProfile = serde_json::from_str(&serde_json::to_string(&uvpr)?)?;
    let result: UserViewProfile = serde_json::from_str(r#"
    {
        "id": 1,
        "title": "Profile 1",
        "description": "Just an example profile",
        "user_prompt_groups": [
            {
                "id": 2,
                "description": "Example 2",
                "user_args": []
            },
            {
                "id": 3,
                "description": "Example 3",
                "user_args": [
                    {
                        "id": 2,
                        "prompt": "Prompt for file",
                        "default": null,
                        "choice_fixed": true,
                        "choices": null
                    }
                ]
            },
            {
                "id": 4,
                "description": "Example 4",
                "user_args": [
                    {
                        "id": 3,
                        "prompt": "Prompt for alternative file",
                        "default": null,
                        "choice_fixed": true,
                        "choices": null
                    }
                ]
            },
            {
                "id": 5,
                "description": "",
                "user_args": []
            },
            {
                "id": 6,
                "description": "Just to have more arguments",
                "user_args": [
                    {
                        "id": 6,
                        "prompt": "First question in this group",
                        "default": null,
                        "choice_fixed": false,
                        "choices": []
                    },
                    {
                        "id": 7,
                        "prompt": "Second question in this group",
                        "default": null,
                        "choice_fixed": false,
                        "choices": []
                    },
                    {
                        "id": 8,
                        "prompt": "Final multiple choice",
                        "default": null,
                        "choice_fixed": false,
                        "choices": [
                            ["Choice A", false],
                            ["Choice B", false],
                            ["Choice C", false]
                        ]
                    }
                ]
            }
        ]
    }
    "#)?;

    assert_eq!(uvp, result);

    // now try to apply a profile to an exposure
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let efc = exposure.create_file("if1").await?;
    let exposure_file_id = efc
        .exposure_file()
        .id();

    platform.mc_platform.set_ef_vttprofile(
        exposure_file_id,
        vttp,
    ).await?;

    let efvttsc = efc.build_vttc().await?;
    let upgr = efvttsc.create_user_prompt_groups().await?;

    let uvg: Vec<UserPromptGroup> = serde_json::from_str(&serde_json::to_string(&upgr)?)?;
    let uvg_result: Vec<UserPromptGroup> = serde_json::from_str(r#"
    [
        {
            "id": 2,
            "description": "Example 2",
            "user_args": []
        },
        {
            "id": 3,
            "description": "Example 3",
            "user_args": [
                {
                    "id": 2,
                    "prompt": "Prompt for file",
                    "default": null,
                    "choice_fixed": true,
                    "choices": [
                        ["README", false],
                        ["if1", true]
                    ]
                }
            ]
        },
        {
            "id": 4,
            "description": "Example 4",
            "user_args": [
                {
                    "id": 3,
                    "prompt": "Prompt for alternative file",
                    "default": null,
                    "choice_fixed": true,
                    "choices": [
                        ["README", false],
                        ["if1", false]
                    ]
                }
            ]
        },
        {
            "id": 5,
            "description": "",
            "user_args": []
        },
        {
            "id": 6,
            "description": "Just to have more arguments",
            "user_args": [
                {
                    "id": 6,
                    "prompt": "First question in this group",
                    "default": null,
                    "choice_fixed": false,
                    "choices": []
                },
                {
                    "id": 7,
                    "prompt": "Second question in this group",
                    "default": null,
                    "choice_fixed": false,
                    "choices": []
                },
                {
                    "id": 8,
                    "prompt": "Final multiple choice",
                    "default": null,
                    "choice_fixed": false,
                    "choices": [
                        ["Choice A", false],
                        ["Choice B", false],
                        ["Choice C", false]
                    ]
                }
            ]
        }
    ]
    "#)?;

    assert_eq!(uvg, uvg_result);

    Ok(())
}

#[async_std::test]
async fn test_exposure_file_view_task_sync() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let vtts = make_example_view_task_templates(&platform).await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let efc = exposure.create_file("if1").await?;
    let exposure_file_id = efc
        .exposure_file()
        .id();

    ExposureTaskTemplateBackend::set_file_templates(
        platform.mc_platform.as_ref(),
        exposure_file_id,
        &[vtts[0]],
    ).await?;
    let efvttsc = efc.build_vttc().await?;
    let user_input = UserInputMap::from([
        (1, "Example answer".to_string()),
    ]);

    let tasks = efvttsc.create_tasks_from_input(&user_input)?;
    let result = efc.process_vttc_tasks(tasks).await?;
    let (_, task_id) = result[0];
    let efvb: &dyn ExposureFileViewBackend = platform.mc_platform.as_ref();
    let id = efvb.select_id_by_task_id(task_id).await?;
    assert_eq!(exposure_file_id, id);

    // this new set of tasks will invalidate the first set of queued
    // tasks for the exposure file view
    let tasks = efvttsc.create_tasks_from_input(&user_input)?;
    efc.process_vttc_tasks(tasks).await?;
    let id = efvb.select_id_by_task_id(task_id).await;
    assert!(id.is_err());

    // TODO need a test that "runs" the last task and have a new task be
    // queued up right after that and have it fail to update the view.

    Ok(())
}

#[async_std::test]
async fn test_exposure_file_view_task_run_view_key_success() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let vtts = make_example_view_task_templates(&platform).await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let efc = exposure.create_file("if1").await?;

    ExposureTaskTemplateBackend::set_file_templates(
        platform.mc_platform.as_ref(),
        efc.exposure_file().id(),
        &[vtts[0]],
    ).await?;
    let efvttsc = efc.build_vttc().await?;
    let user_input = UserInputMap::from([
        (1, "Example answer".to_string()),
    ]);

    let tasks = efvttsc.create_tasks_from_input(&user_input)?;
    let result = efc.process_vttc_tasks(tasks).await?;
    let (exposure_file_view_id, task_id) = result[0];

    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), None);

    // spawn a task
    let mut task = platform.tm_platform.as_ref()
        .start_task()
        .await?
        .expect("task was queued");
    assert_eq!(task.id(), task_id);
    task.run(12345).await?;

    let result = platform.complete_task(task, 0).await?;
    assert!(result);

    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), Some("example_view1"));

    Ok(())
}

#[async_std::test]
async fn test_exposure_file_view_task_run_task_fail() -> anyhow::Result<()> {
    // TODO find out why there may be spurious FOREIGN KEY constraint
    // failure in this test - probably due to certain hard-coded ids.
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let vtts = make_example_view_task_templates(&platform).await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let efc = exposure.create_file("if1").await?;

    ExposureTaskTemplateBackend::set_file_templates(
        platform.mc_platform.as_ref(),
        efc.exposure_file().id(),
        &[vtts[0]],
    ).await?;
    let efvttsc = efc.build_vttc().await?;
    let user_input = UserInputMap::from([
        (1, "Example answer".to_string()),
    ]);

    let tasks = efvttsc.create_tasks_from_input(&user_input)?;
    let result = efc.process_vttc_tasks(tasks).await?;
    let (exposure_file_view_id, task_id) = result[0];

    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), None);

    // spawn a task
    let mut task = platform.tm_platform.as_ref()
        .start_task()
        .await?
        .expect("task was queued");
    assert_eq!(task.id(), task_id);
    // pretend we ran it and complete it
    task.run(12345).await?;

    let result = platform.complete_task(task, 1).await?;
    assert!(!result);

    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), None);

    Ok(())
}

#[async_std::test]
async fn test_exposure_file_view_task_run_task_stale() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let vtts = make_example_view_task_templates(&platform).await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let efc = exposure.create_file("if1").await?;

    ExposureTaskTemplateBackend::set_file_templates(
        platform.mc_platform.as_ref(),
        efc.exposure_file().id(),
        &[vtts[0]],
    ).await?;
    let efvttsc = efc.build_vttc().await?;
    let user_input = UserInputMap::from([
        (1, "Example answer".to_string()),
    ]);

    let tasks = efvttsc.create_tasks_from_input(&user_input)?;
    let result = efc.process_vttc_tasks(tasks).await?;
    let (exposure_file_view_id, task_id) = result[0];

    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), None);

    // spawn a task
    let mut task = platform.tm_platform.as_ref()
        .start_task()
        .await?
        .expect("task was queued");
    assert_eq!(task.id(), task_id);

    // record another task being queued.
    efc.process_vttc_tasks(
        efvttsc.create_tasks_from_input(&user_input)?,
    ).await?;

    // now run the task, that is now stale...
    task.run(12345).await?;
    // ... even if it was started later.
    let later_task = platform.tm_platform.as_ref()
        .start_task()
        .await?
        .expect("task was queued");

    let result = platform.complete_task(task, 0).await?;
    assert!(!result);
    // That shouldn't set the view just yet.
    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), None);

    assert!(platform.complete_task(later_task, 0).await?);
    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), Some("example_view1"));

    Ok(())
}

// this tests usage of registries that reference values secured against
// end user access, such as the working_dir and actual location of the
// underlying workspace.
#[async_std::test]
async fn test_hidden_registries() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let mut exposure_basedir = PathBuf::new();
    exposure_basedir.push(platform.data_root());
    exposure_basedir.push("exposure");
    exposure_basedir.push("1");
    let exposure_file_basedir = exposure_basedir.join("1");
    let target = "branch/leaf/z1";
    let workspace_filepath = exposure_basedir.join("files").join(target)
        .display()
        .to_string();
    let working_dir = exposure_file_basedir
        .join("default_hidden")
        .display()
        .to_string();

    let vtts = make_example_view_task_templates(&platform).await?;
    let exposure = platform.create_exposure(
        1,
        "8dd710b6b5cf607711bc44f5ca0204565bf7cc35",
    ).await?;
    let efc = exposure.create_file(target).await?;
    let exposure_file_id = efc
        .exposure_file()
        .id();
    ExposureTaskTemplateBackend::set_file_templates(
        platform.mc_platform.as_ref(),
        exposure_file_id,
        &[vtts[4]],
    ).await?;

    let efvttsc = efc.build_vttc().await?;
    let user_arg_refs = efvttsc.create_user_arg_refs().await?;
    assert_eq!(user_arg_refs.len(), 0);

    let user_input = UserInputMap::from([]);
    let tasks = efvttsc.create_tasks_from_input(&user_input)?
        .into_iter()
        .map(<(i64, Task)>::from)
        .collect::<Vec<_>>();

    let answers: Vec<(i64, Task)> = serde_json::from_str(&format!(r#"
    [
        [5, {{
            "id": 0,
            "task_template_id": 6,
            "bin_path": "/usr/local/bin/hidden_example",
            "pid": null,
            "created_ts": 0,
            "start_ts": null,
            "stop_ts": null,
            "exit_status": null,
            "basedir": "{working_dir}",
            "args": [
                {{
                    "id": 0,
                    "task_id": 0,
                    "arg": "--workspace_file_path={workspace_filepath}"
                }},
                {{
                    "id": 0,
                    "task_id": 0,
                    "arg": "--working_dir={working_dir}"
                }}
            ]
        }}]
    ]
    "#))?;
    assert_eq!(&answers, &tasks);


    Ok(())
}

#[async_std::test]
async fn test_task_executor_ctrl() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let vtts = make_example_runnable_task_templates(&platform).await?;

    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let efc = exposure.create_file("if1").await?;
    let exposure_file_id = efc
        .exposure_file()
        .id();
    ExposureTaskTemplateBackend::set_file_templates(
        platform.mc_platform.as_ref(),
        exposure_file_id,
        &[vtts[0]],
    ).await?;
    let efvttsc = efc.build_vttc().await?;
    let user_input = UserInputMap::from([]);
    let tasks = efvttsc.create_tasks_from_input(&user_input)?;
    let result = efc.process_vttc_tasks(tasks).await?;
    let (exposure_file_view_id, _) = result[0];

    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), None);

    let task_executor_ctrl = platform.start_task().await?
        .expect("a task is queued");
    let (code, result) = task_executor_ctrl.execute().await?;
    assert_eq!(code, 0);
    assert_eq!(result, true);

    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), Some("sentinel_0"));

    Ok(())
}

#[async_std::test]
async fn test_task_executor_ctrl_queued_extra() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let vtts = make_example_runnable_task_templates(&platform).await?;

    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let efc = exposure.create_file("if1").await?;
    let exposure_file_id = efc
        .exposure_file()
        .id();
    ExposureTaskTemplateBackend::set_file_templates(
        platform.mc_platform.as_ref(),
        exposure_file_id,
        &[vtts[0]],
    ).await?;
    let efvttsc = efc.build_vttc().await?;
    let user_input = UserInputMap::from([]);
    let tasks = efvttsc.create_tasks_from_input(&user_input)?;
    let result = efc.process_vttc_tasks(tasks).await?;
    let (exposure_file_view_id, _) = result[0];

    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), None);

    let task_executor_ctrl = platform.start_task().await?
        .expect("a task is queued");

    // now mess it up by queuing and processing a new set of tasks
    efc.process_vttc_tasks(
        efvttsc.create_tasks_from_input(&user_input)?
    ).await?;

    let (code, result) = task_executor_ctrl.execute().await?;
    assert_eq!(code, 0);
    assert_eq!(result, false);

    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), None);

    Ok(())
}

#[async_std::test]
async fn test_task_executor_ctrl_task_failure_then_success() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let vtts = make_example_runnable_task_templates(&platform).await?;

    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let efc = exposure.create_file("if1").await?;
    let exposure_file_id = efc
        .exposure_file()
        .id();
    ExposureTaskTemplateBackend::set_file_templates(
        platform.mc_platform.as_ref(),
        exposure_file_id,
        &[vtts[1]],
    ).await?;
    let efvttsc = efc.build_vttc().await?;
    let user_input = UserInputMap::from([
        (1, "42".to_string()),
    ]);
    let tasks = efvttsc.create_tasks_from_input(&user_input)?;
    let result = efc.process_vttc_tasks(tasks).await?;
    let (exposure_file_view_id, _) = result[0];

    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), None);

    let task_executor_ctrl = platform.start_task().await?
        .expect("a task is queued");
    let (code, result) = task_executor_ctrl.execute().await?;
    assert_eq!(code, 42);
    assert_eq!(result, false);

    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), None);

    // now simulate a correction to a successful run
    let user_input = UserInputMap::from([
        (1, "0".to_string()),
    ]);
    let tasks = efvttsc.create_tasks_from_input(&user_input)?;
    let result = efc.process_vttc_tasks(tasks).await?;
    assert_eq!(result.len(), 1);
    let task_executor_ctrl = platform.start_task().await?
        .expect("a task is queued");
    let (code, result) = task_executor_ctrl.execute().await?;
    assert_eq!(code, 0);
    assert_eq!(result, true);
    let efv = platform.mc_platform.as_ref()
        .get_exposure_file_view(exposure_file_view_id)
        .await?;
    assert_eq!(efv.view_key(), Some("exit_code_0"));

    Ok(())
}

#[async_std::test]
async fn test_multiple_exposure_files() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    let efc1 = exposure.create_file("if1").await?;
    let efc2 = exposure.create_file("README").await?;

    assert_eq!(efc1.exposure_file().id(), 1);
    assert_eq!(efc2.exposure_file().id(), 2);

    Ok(())
}

#[async_std::test]
async fn test_exposure_file_registry() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let exposure = platform.create_exposure(
        1,
        "42845247d1a2af1bf5a0f09c85e254ba78992c2f",
    ).await?;
    let efc = exposure.create_file("if1").await?;
    let registry: PreparedChoiceRegistry = (&efc).try_into()?;
    let files: UserChoiceRefs = registry.lookup("files")
        .expect("has files registry")
        .into();
    assert_eq!(files.as_slice(), &[
        UserChoiceRef("README", false),
        UserChoiceRef("branch/alpha", false),
        UserChoiceRef("branch/beta", false),
        UserChoiceRef("if1", false),
    ]);

    let files_default: UserChoiceRefs = registry.lookup("files_default")
        .expect("has files_default registry")
        .into();
    assert_eq!(files_default.as_slice(), &[
        UserChoiceRef("README", false),
        UserChoiceRef("branch/alpha", false),
        UserChoiceRef("branch/beta", false),
        UserChoiceRef("if1", true),
    ]);

    Ok(())
}

#[test]
fn test_send_sync_ctrl() {
    fn is_send_sync<T: Send + Sync>() { }
    is_send_sync::<pmrctrl::handle::ExposureCtrl<SqliteBackend, SqliteBackend>>();
    is_send_sync::<pmrctrl::handle::ExposureFileCtrl<SqliteBackend, SqliteBackend>>();
}
