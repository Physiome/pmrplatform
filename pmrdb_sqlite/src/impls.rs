use async_trait::async_trait;
// use pmrdb::{MigrationProfile, SqlxSource};
use pmrcore::platform::{ACPlatform, MCPlatform,PlatformBuilder, PlatformUrl, TMPlatform};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::SqliteBackend;

impl PlatformUrl for SqliteBackend {
    fn url(&self) -> &str {
        self.url.as_ref()
    }
}

impl SqliteBackend {
    pub async fn connect(url: &str) -> Result<SqliteBackend, sqlx::Error> {
        let pool = SqlitePool::connect(url).await?;
        Ok(SqliteBackend {
            pool: Arc::new(pool),
            url: url.to_string(),
        })
    }

    pub async fn migrate_ac(self) -> Result<Self, sqlx::Error> {
        sqlx::migrate!("migrations/pmrac").run(&*self.pool).await?;
        Ok(self)
    }

    pub async fn migrate_mc(self) -> Result<Self, sqlx::Error> {
        sqlx::migrate!("migrations/pmrapp").run(&*self.pool).await?;
        Ok(self)
    }

    pub async fn migrate_tm(self) -> Result<Self, sqlx::Error> {
        sqlx::migrate!("migrations/pmrtqs").run(&*self.pool).await?;
        Ok(self)
    }
}

#[async_trait]
impl PlatformBuilder for SqliteBackend {
    async fn ac(url: &str) -> Result<impl ACPlatform, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let backend = SqliteBackend::connect(url).await
            .map_err(Box::new)?
            .migrate_ac()
            .await
            .map_err(Box::new)?;
        Ok(backend)
    }

    async fn mc(url: &str) -> Result<impl MCPlatform, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let backend = SqliteBackend::connect(url).await
            .map_err(Box::new)?
            .migrate_mc()
            .await
            .map_err(Box::new)?;
        Ok(backend)
    }

    async fn tm(url: &str) -> Result<impl TMPlatform, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let backend = SqliteBackend::connect(url).await
            .map_err(Box::new)?
            .migrate_tm()
            .await
            .map_err(Box::new)?;
        Ok(backend)
    }
}

mod ac;
mod alias;

mod exposure;
mod exposure_file;
mod exposure_file_profile;
mod exposure_file_view;
mod exposure_file_view_task;
mod exposure_file_view_task_template;

mod idgen;

mod profile;

mod workspace;
mod workspace_sync;
mod workspace_tag;

mod task;
mod task_template;

mod default_impl {
    use pmrcore::platform::{
        DefaultACPlatform,
        DefaultMCPlatform,
        DefaultTMPlatform,
    };
    use crate::SqliteBackend;

    impl DefaultACPlatform for SqliteBackend {}
    impl DefaultMCPlatform for SqliteBackend {}
    impl DefaultTMPlatform for SqliteBackend {}
}

