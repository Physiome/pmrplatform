use pmrcore::workspace::traits::WorkspaceBackend;
use pmrctrl::platform::Platform;
use pmrmodel::backend::db::{
    Profile,
    SqliteBackend,
};
use tempfile::TempDir;

use crate::repo::create_repodata;

pub async fn create_blank_sqlite_platform<'a>() -> anyhow::Result<(
    TempDir,
    Platform<'a, SqliteBackend, SqliteBackend>,
)> {
    let tempdir = TempDir::new()?;
    let platform = Platform::new(
        SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Profile::Pmrapp)
            .await?,
        SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(Profile::Pmrtqs)
            .await?,
        tempdir.path().to_path_buf(),
    );
    Ok((tempdir, platform))
}

pub async fn create_sqlite_platform<'a>() -> anyhow::Result<(
    TempDir,
    Platform<'a, SqliteBackend, SqliteBackend>,
)> {
    let (tempdir, _, _, _) = create_repodata();
    let mc = SqliteBackend::from_url("sqlite::memory:")
        .await?
        .run_migration_profile(Profile::Pmrapp)
        .await?;
    let tm = SqliteBackend::from_url("sqlite::memory:")
        .await?
        .run_migration_profile(Profile::Pmrtqs)
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

    let platform = Platform::new(mc, tm, tempdir.path().to_path_buf());
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
