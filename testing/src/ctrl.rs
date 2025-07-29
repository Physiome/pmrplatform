use pmrac::platform::Builder as ACPlatformBuilder;
use pmrctrl::platform::Platform;
use pmrdb::Backend;
use tempfile::TempDir;

use crate::repo::inject_repodata;

pub async fn create_blank_sqlite_platform() -> anyhow::Result<(
    TempDir,
    Platform,
)> {
    create_blank_platform("sqlite::memory:").await
}

async fn create_blank_platform(url: &str) -> anyhow::Result<(
    TempDir,
    Platform,
)> {
    let tempdir = TempDir::new()?;
    let repo_root = tempdir.path().join("repo").to_path_buf();
    let data_root = tempdir.path().join("data").to_path_buf();
    std::fs::create_dir_all(&repo_root)?;
    std::fs::create_dir_all(&data_root)?;

    let platform = Platform::new(
        ACPlatformBuilder::new()
            .boxed_ac_platform(
                Backend::ac(url.into()).await
                    .map_err(anyhow::Error::from_boxed)?,
            )
            .build(),
        Backend::mc(url.into()).await
            .map_err(anyhow::Error::from_boxed)?
            .into(),
        Backend::tm(url.into()).await
            .map_err(anyhow::Error::from_boxed)?
            .into(),
        data_root,
        repo_root,
    );
    Ok((tempdir, platform))
}

pub async fn create_sqlite_platform() -> anyhow::Result<(
    TempDir,
    Platform,
)> {
    let (tempdir, platform) = create_blank_sqlite_platform().await?;
    inject_repodata(platform.repo_root());

    let wb = platform.mc_platform.as_ref();
    wb.add_workspace(
        "https://models.example.com/import1/".into(),
        "import1".into(),
        "".into(),
    ).await?;
    wb.add_workspace(
        "https://models.example.com/import2/".into(),
        "import2".into(),
        "".into(),
    ).await?;
    wb.add_workspace(
        "https://models.example.com/repodata/".into(),
        "repodata".into(),
        "".into(),
    ).await?;

    Ok((tempdir, platform))
}

#[cfg(test)]
mod testing {
    use super::*;

    #[async_std::test]
    async fn smoke_test_create_platform() -> anyhow::Result<()> {
        create_sqlite_platform().await?;
        create_blank_sqlite_platform().await?;
        Ok(())
    }
}
