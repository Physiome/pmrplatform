use pmrcore::exposure::traits::Exposure;

// switch the comments between these two identical code for an interesting
// compiler issue - see `pmrctrl::testing` for details
// use test_pmr::ctrl::create_sqlite_platform;
use crate::testing::create_sqlite_platform;

use crate::registry::make_choice_registry;

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
                    "choice_source": "file",
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
    let file = exposure.create_file("if1").await?;

    // Create the task

    // 1. Load the selected task template
    let template = platform.get_view_task_template(template_id).await?;
    let s = serde_json::to_string(&template)?;

    let registry = make_choice_registry(&exposure);
    println!("{s}");
    // 2. Create the choice registry
    // 3. generate the questions
    // 4. apply the arguments with task builder, create the task.

    Ok(())
}
