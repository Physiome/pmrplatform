use pmrac::{
    Platform,
    platform::Builder,
};
use pmrmodel::backend::db::{
    MigrationProfile,
    SqliteBackend,
};

pub async fn create_sqlite_backend() -> anyhow::Result<SqliteBackend> {
    Ok(SqliteBackend::from_url("sqlite::memory:")
        .await?
        .run_migration_profile(MigrationProfile::Pmrac)
        .await?)
}

pub async fn create_sqlite_platform(purge: bool) -> anyhow::Result<Platform> {
    let platform = Builder::new()
        .ac_platform(create_sqlite_backend().await?)
        .password_autopurge(purge)
        .build();
    Ok(platform)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn smoke_test_create_platform() -> anyhow::Result<()> {
        create_sqlite_platform(true).await?;
        create_sqlite_platform(false).await?;
        Ok(())
    }
}
