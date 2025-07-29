use pmrac::{
    Platform,
    platform::Builder,
};
use pmrcore::platform::ACPlatform;
use pmrdb::Backend;

pub async fn create_sqlite_backend() -> anyhow::Result<Box<dyn ACPlatform>> {
    Ok(Backend::ac("sqlite::memory:".into())
        .await
        .map_err(anyhow::Error::from_boxed)?)
}

pub async fn create_sqlite_platform(purge: bool) -> anyhow::Result<Platform> {
    let platform = Builder::new()
        .boxed_ac_platform(create_sqlite_backend().await?)
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
