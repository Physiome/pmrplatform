use pmrac::Platform;
use pmrmodel::backend::db::{
    MigrationProfile,
    SqliteBackend,
};

pub async fn create_sqlite_platform() -> anyhow::Result<Platform> {
    let platform = Platform::new(
        SqliteBackend::from_url("sqlite::memory:")
            .await?
            .run_migration_profile(MigrationProfile::Pmrac)
            .await?,
    );
    Ok(platform)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_std::test]
    async fn smoke_test_create_platform() -> anyhow::Result<()> {
        create_sqlite_platform().await?;
        Ok(())
    }
}
