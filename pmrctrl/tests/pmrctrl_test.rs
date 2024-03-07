use pmrcore::{
    exposure::{
        task::traits::{
            ExposureTaskTemplateBackend,
            ExposureTaskBackend,
        },
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
    backend::db::SqliteBackend,
    model::task_template::{
        TaskBuilder,
        UserArgBuilder,
        UserInputMap,
    },
    registry::{
        ChoiceRegistry,
        ChoiceRegistryCache,
        PreparedChoiceRegistry,
    },
};
use pmrctrl::{
    handle::ViewTaskTemplatesCtrl,
    platform::Platform,
};
use std::{
    path::PathBuf,
    fs::read_to_string,
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

    let efctrl = exposure.ctrl_file(ef_ref).await?;
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
    // let new_task = TaskBackend::gets_task(&platform.tm_platform, 1);

    // TODO actually tying the task back to the exposure file and thus
    // the appropriate view - this test really is a current proof of
    // concept while figuring stuff out.

    Ok(())
}

async fn make_example_view_task_templates<'a, M, T>(
    platform: &'a Platform<'a, M, T>
) -> anyhow::Result<Vec<i64>>
where
    M: MCPlatform + Sized + Sync,
    T: TMPlatform + Sized + Sync,
{
    use pmrcore::task_template::traits::TaskTemplateBackend;
    // force insertion of a dummy task template that should shift the
    // id for the ExposureFileTaskTemplate vs TaskTemplate.
    let ttb: &dyn TaskTemplateBackend = &platform.tm_platform;
    ttb.add_task_template("/bin/dummy", "1.0.0").await?;

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
    let vtts = make_example_view_task_templates(&platform).await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    // this is now needed to avoid borrow checker getting confused about
    // the order of which vttc is freed (before exposure_file_ctrl, ensure
    // that we drop that.
    let exposure_file_id = {
        let exposure_file_ctrl = exposure.create_file("if1").await?;
        let exposure_file_ref = exposure_file_ctrl.exposure_file();
        exposure_file_ref.id()
    };

    let vttc = platform.get_file_templates_for_exposure_file(exposure_file_id).await?;
    let vtt: &ViewTaskTemplates = (&vttc).into();
    assert_eq!(vtt.len(), 0);

    ExposureTaskTemplateBackend::set_file_templates(
        &platform.mc_platform,
        exposure_file_id,
        [vtts[0]].into_iter(),
    ).await?;
    let vttc = platform.get_file_templates_for_exposure_file(exposure_file_id).await?;
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

    ExposureTaskTemplateBackend::set_file_templates(
        &platform.mc_platform,
        exposure_file_id,
        [vtts[1], vtts[2]].into_iter(),
    ).await?;
    let vttc = platform.get_file_templates_for_exposure_file(exposure_file_id).await?;
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
    // these variables must be defined here to avoid borrow checker from
    // being angry about these being dropped while still borrowed as the
    // destructor for the ExposureFileCtrl (declared later) runs.
    let efvttsc: ViewTaskTemplatesCtrl<SqliteBackend, SqliteBackend>;
    let user_input: UserInputMap;

    let (_reporoot, platform) = create_sqlite_platform().await?;
    let mut exposure_file_basedir = PathBuf::new();
    exposure_file_basedir.push(platform.data_root());
    exposure_file_basedir.push("exposure");
    exposure_file_basedir.push("1");
    exposure_file_basedir.push("1");
    let exposure_file_basedir = exposure_file_basedir.display();

    let vtts = make_example_view_task_templates(&platform).await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    // this apparently triggers the destructor failure
    let efc = exposure.create_file("if1").await?;
    let exposure_file_id = efc
        .exposure_file()
        .id();

    ExposureTaskTemplateBackend::set_file_templates(
        &platform.mc_platform,
        exposure_file_id,
        [vtts[0], vtts[3]].into_iter(),
    ).await?;
    assert_eq!(vtts[0], 1);
    assert_eq!(vtts[3], 4);
    efvttsc = platform.get_file_templates_for_exposure_file(exposure_file_id).await?;

    // If the ExposureFileViewTemplatesCtrl is borrowed, the borrow
    // checker will become angry if the declarations weren't made at
    // the top and if the ExposureFileCtrl isn't dropped here...
    // drop(efc);

    // ... before efvttsc gets borrowed here.
    let user_arg_refs = efvttsc.create_user_arg_refs().await?;
    let user_args: Vec<UserArg> = user_arg_refs.iter()
        .map(|a| a.into())
        .collect();
    assert_eq!(user_arg_refs.len(), 2);
    assert_eq!(user_args[0].id, 1);
    assert_eq!(user_args[0].prompt, "Example prompt");
    assert_eq!(user_args[1].id, 3);
    assert_eq!(user_args[1].prompt, "Prompt for alternative file");
    // TODO test for alternative ID remaps via manual deletes/updates to the
    // underlying linkage between ViewTaskTemplate and TaskTemplate

    user_input = UserInputMap::from([
        (1, "Example answer".to_string()),
        (3, "README".to_string()),
    ]);

    let tasks = efvttsc.create_tasks_from_input(&user_input).await?
        .into_iter()
        .map(<(i64, Task)>::from)
        .collect::<Vec<_>>();

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
            "basedir": "{exposure_file_basedir}",
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
            "basedir": "{exposure_file_basedir}",
            "args": [
                {{
                    "id": 0,
                    "task_id": 0,
                    "arg": "--file2=README"
                }}
            ]
        }}]
    ]
    "#))?;
    assert_eq!(&answers, &tasks);

    // since the one above was consumed for inspection, repeat that call
    // and pass the new one for processing.
    let tasks = efvttsc.create_tasks_from_input(&user_input).await?;
    let result = efc.process_vttc_tasks(tasks).await?;
    assert_eq!(result.len(), 2);

    // TODO finalize the ExposureFileViewTask handling via the platform
    // but for now just use the underlying and find out whether the
    // tasks have been correctly queued.

    let etb: &dyn ExposureTaskBackend = &platform.mc_platform;
    let et1 = etb.select_task_for_view(result[0]).await?
        .expect("not none");
    assert_eq!(et1.id, 1);
    assert_eq!(et1.exposure_file_view_id, 1);
    assert_eq!(et1.view_task_template_id, 1);
    assert_eq!(et1.ready, false);

    let tb: &dyn TaskBackend = &platform.tm_platform;
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
        "basedir": "{exposure_file_basedir}",
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
