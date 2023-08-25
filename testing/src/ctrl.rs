use pmrctrl::platform::Platform;
use pmrmodel::backend::db::{
    Profile,
    SqliteBackend,
};
use tempfile::TempDir;

use crate::repo::create_repodata;

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

    let platform = Platform::new(mc, tm, tempdir.path().to_path_buf());
    Ok((tempdir, platform))
}
