use test_pmr::ctrl::create_sqlite_platform;

#[async_std::test]
async fn test_platform_core() -> anyhow::Result<()> {
    let (tempdir, platform) = create_sqlite_platform().await?;
    Ok(())
}
