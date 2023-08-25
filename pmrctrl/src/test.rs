use test_pmr::ctrl::create_sqlite_platform;

#[async_std::test]
async fn test_platform_core() -> anyhow::Result<()> {
    let (_reporoot, platform) = create_sqlite_platform().await?;
    let exposure = platform.create_exposure(
        1,
        "083b775d81ec9b66796edbbdce4d714bb2ddc355",
    ).await?;
    assert_eq!(exposure.list_files()?, &["README", "if1"]);
    assert_eq!(exposure.list_exposure_files().await?.len(), 0);
    let _file = exposure.create_file("if1").await?;
    // this is cached from above.
    assert_eq!(exposure.list_exposure_files().await?.len(), 0);

    // load a new copy
    // TODO figure out how/what to expose inner
    // let exposure = platform.get_exposure(exposure.inner.id()).await?;
    // TODO figure out lifetime issues here.
    // let ex2 = platform.get_exposure(1).await?;
    // let files = ex2.list_exposure_files().await?;
    // assert_eq!(files, &["if1"]);
    Ok(())
}
