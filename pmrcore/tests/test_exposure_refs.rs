use mockall::predicate::eq;
use pmrcore::{
    error::BackendError,
    exposure::{
        ExposureFileView,
        traits::{
            ExposureFileView as _,
            ExposureFileViewBackend,
        },
    },
    platform::MCPlatform,
};
use test_pmr::core::MockPlatform;

// like the other instance, it would be good if the testing crate and
// this crate that's being tested somehow also pretend the crate imports
// are identical, but since one is identified as crate and the other via
// its fully qualified import path, this external test will need to be
// done instead of the slightly more straightforward module level tests.
#[async_std::test]
async fn test_update_efvref_view_key() -> anyhow::Result<()> {
    let mut platform = MockPlatform::new();
    platform.expect_exposure_file_view_get_id()
        .times(1)
        .with(eq(1))
        .returning(move |_| Ok(ExposureFileView {
            id: 1,
            exposure_file_view_task_id: None,
            view_task_template_id: 1,
            exposure_file_id: 1,
            view_key: None,
            updated_ts: 1234567890,
        }));
    platform.expect_exposure_file_view_update_view_key()
        .times(1)
        .withf(|a, b| a == &1 && b == &Some("new_view"))
        .returning(move |_, _| Ok(true));
    platform.expect_exposure_file_view_update_view_key()
        .times(1)
        .withf(|a, b| a == &1 && b == &Some("error_view"))
        .returning(move |_, _| Err(BackendError::Unknown));
    platform.expect_exposure_file_view_update_view_key()
        .times(1)
        .withf(|a, b| a == &1 && b == &Some("view_gone"))
        .returning(move |_, _| Ok(false));

    let mut efv = platform.get_exposure_file_view(1).await?;
    assert_eq!(efv.view_key(), None);

    efv.update_view_key(Some("new_view")).await?;
    assert_eq!(efv.view_key(), Some("new_view"));

    // now if the update fails, no update issued.
    assert!(efv.update_view_key(Some("error_view")).await.is_err());
    assert_eq!(efv.view_key(), Some("new_view"));

    // if the underlying is gone, no update issued again.
    let result = efv.update_view_key(Some("view_gone")).await;
    assert!(matches!(
        result,
        Err(BackendError::AppInvariantViolation(msg))
            if msg == "Underlying ExposureFileView is gone (id: 1)"
    ));
    assert_eq!(efv.view_key(), Some("new_view"));

    Ok(())
}

#[async_std::test]
async fn test_update_efvref_efv_task_id() -> anyhow::Result<()> {
    let mut platform = MockPlatform::new();
    platform.expect_exposure_file_view_get_id()
        .times(1)
        .with(eq(1))
        .returning(move |_| Ok(ExposureFileView {
            id: 1,
            exposure_file_view_task_id: None,
            view_task_template_id: 1,
            exposure_file_id: 1,
            view_key: None,
            updated_ts: 1234567890,
        }));
    platform.expect_exposure_file_view_update_exposure_file_view_task_id()
        .times(1)
        .withf(|a, b| a == &1 && b == &Some(1))
        .returning(move |_, _| Ok(true));
    platform.expect_exposure_file_view_update_exposure_file_view_task_id()
        .times(1)
        .withf(|a, b| a == &1 && b == &Some(2))
        .returning(move |_, _| Err(BackendError::Unknown));
    platform.expect_exposure_file_view_update_exposure_file_view_task_id()
        .times(1)
        .withf(|a, b| a == &1 && b == &Some(3))
        .returning(move |_, _| Ok(false));

    let mut efv = platform.get_exposure_file_view(1).await?;
    assert_eq!(efv.exposure_file_view_task_id(), None);

    efv.update_exposure_file_view_task_id(Some(1)).await?;
    assert_eq!(efv.exposure_file_view_task_id(), Some(1));

    // now if the update fails, no update issued.
    assert!(efv.update_exposure_file_view_task_id(Some(2)).await.is_err());
    assert_eq!(efv.exposure_file_view_task_id(), Some(1));

    // if the underlying is gone, no update issued again.
    let result = efv.update_exposure_file_view_task_id(Some(3)).await;
    assert!(matches!(
        result,
        Err(BackendError::AppInvariantViolation(msg))
            if msg == "Underlying ExposureFileView is gone (id: 1)"
    ));
    assert_eq!(efv.exposure_file_view_task_id(), Some(1));

    Ok(())
}
