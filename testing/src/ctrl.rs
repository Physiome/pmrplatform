use pmrcore::workspace::traits::WorkspaceBackend;
use pmrctrl::platform::Platform;
use pmrmodel::backend::db::{
    MigrationProfile,
    SqliteBackend,
};
use tempfile::TempDir;

use crate::repo::inject_repodata;

pub async fn create_blank_sqlite_platform() -> anyhow::Result<(
    TempDir,
    Platform,
)> {
    let tempdir = TempDir::new()?;
    let repo_root = tempdir.path().join("repo").to_path_buf();
    let data_root = tempdir.path().join("data").to_path_buf();
    std::fs::create_dir_all(&repo_root)?;
    std::fs::create_dir_all(&data_root)?;

    let platform = Platform::new(
        SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrapp)
            .await?,
        SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrtqs)
            .await?,
        data_root,
        repo_root,
    );
    Ok((tempdir, platform))
}

pub async fn create_sqlite_platform() -> anyhow::Result<(
    TempDir,
    Platform,
)> {
    let tempdir = TempDir::new()?;
    let repo_root = tempdir.path().join("repo").to_path_buf();
    let data_root = tempdir.path().join("data").to_path_buf();
    std::fs::create_dir_all(&repo_root)?;
    std::fs::create_dir_all(&data_root)?;

    inject_repodata(repo_root.as_ref());

    let mc = SqliteBackend::from_url("sqlite::memory:")
        .await?
        .run_migration_profile(MigrationProfile::Pmrapp)
        .await?;
    let tm = SqliteBackend::from_url("sqlite::memory:")
        .await?
        .run_migration_profile(MigrationProfile::Pmrtqs)
        .await?;

    let wb: &dyn WorkspaceBackend = &mc;
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

    let platform = Platform::new(
        mc,
        tm,
        data_root,
        repo_root,
    );
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
