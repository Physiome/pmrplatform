use pmrcore::workspace::traits::WorkspaceBackend;
use pmrmodel::backend::db::{
    Profile,
    SqliteBackend,
};
use tempfile::TempDir;

use crate::platform::Platform;
use test_pmr::repo::create_repodata;

// XXX this is a DIRECT copy of the relevant piece from the testing
// `test_pmr::repo::create_sqlite_platform`, doing so to avoid a
// compilation issue where the re-exported version of the return
// type is different to the expected internal version within this
// `pmrctrl` crate.
// possibly related issue: https://github.com/rust-lang/cargo/issues/8639
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
        Ok(())
    }
}
