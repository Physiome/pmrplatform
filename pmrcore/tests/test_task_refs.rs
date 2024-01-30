use mockall::predicate::eq;
use pmrcore::{
    platform::TMPlatform,
    task::Task,
};

use test_pmr::core::MockPlatform;

#[async_std::test]
async fn test_ref_impls() -> anyhow::Result<()> {
    let mut platform = MockPlatform::new();
    let task_id = 1;
    let task_pid = 123;
    let task_exit = 0;
    platform.expect_start()
        .times(1)
        .with()
        .returning(move || Ok(Some(
            Task {
                id: task_id,
                .. Default::default()
            },
        )));
    platform.expect_run()
        .times(1)
        .with(eq(task_id), eq(task_pid))
        .returning(|_, _| Ok(true));
    platform.expect_complete()
        .times(1)
        .with(eq(task_id), eq(task_exit))
        .returning(|_, _| Ok(true));

    let mut task_ref = platform.start_task()
        .await?
        .expect("task started");
    assert_eq!(task_ref.pid(), None);
    task_ref.run(task_pid).await?;
    assert_eq!(task_ref.pid(), Some(task_pid));

    assert_eq!(task_ref.exit_status(), None);
    task_ref.complete(task_exit).await?;
    assert_eq!(task_ref.exit_status(), Some(task_exit));

    assert_eq!(task_ref.into_inner(), Task {
        id: 1,
        pid: Some(123),
        exit_status: Some(0),
        .. Default::default()
    });
    Ok(())
}
