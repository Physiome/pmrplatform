use async_trait::async_trait;
use pmrcore::platform::{ACPlatform, ConnectorOption, MCPlatform, PlatformConnector, PlatformUrl, TMPlatform};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::sync::Arc;

use crate::SqliteBackend;

impl PlatformUrl for SqliteBackend {
    fn url(&self) -> &str {
        self.url.as_ref()
    }
}

impl SqliteBackend {
    pub async fn connect(opts: ConnectorOption) -> Result<SqliteBackend, sqlx::Error> {
        if opts.auto_create_db && !Sqlite::database_exists(&opts.url).await.unwrap_or(false) {
            log::warn!("sqlite database {} does not exist; creating...", &opts.url);
            Sqlite::create_database(&opts.url).await?
        }

        let pool = SqlitePool::connect(&opts.url).await?;
        Ok(SqliteBackend {
            pool: Arc::new(pool),
            url: opts.url,
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
impl PlatformConnector for SqliteBackend {
    async fn ac(opts: ConnectorOption) -> Result<impl ACPlatform, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let backend = SqliteBackend::connect(opts).await
            .map_err(Box::new)?
            .migrate_ac()
            .await
            .map_err(Box::new)?;
        Ok(backend)
    }

    async fn mc(opts: ConnectorOption) -> Result<impl MCPlatform, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let backend = SqliteBackend::connect(opts).await
            .map_err(Box::new)?
            .migrate_mc()
            .await
            .map_err(Box::new)?;
        Ok(backend)
    }

    async fn tm(opts: ConnectorOption) -> Result<impl TMPlatform, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let backend = SqliteBackend::connect(opts).await
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

// TODO eventually if we need to override certain implementation to better
// optimize selection:
//
// mod specialized {
//     use async_trait::async_trait;
//     use pmrcore::{
//         alias::AliasEntries,
//         error::BackendError,
//         platform::MCPlatform,
//         workspace::WorkspaceRef,
//     };
//
//     #[async_trait]
//     impl MCPlatform for SqliteBackend {
//         fn as_dyn(&self) -> &dyn MCPlatform {
//             self
//         }
//         async fn list_aliased_workspaces<'a>(
//             &'a self,
//         ) -> Result<AliasEntries<WorkspaceRef<'a>>, BackendError> {
//             todo!()
//         }
//     }
// }

// For testing unified usage/traits
#[cfg(test)]
pub(crate) mod tests {
    use pmrcore::{
        platform::{
            MCPlatform,
            PlatformConnector,
        },
        workspace::Workspace,
    };
    use crate::SqliteBackend;

    #[async_std::test]
    async fn create_aliased_workspace() -> anyhow::Result<()> {
        let backend = SqliteBackend::mc("sqlite::memory:".into())
            .await
            .map_err(anyhow::Error::from_boxed)?;
        let entry = backend.create_aliased_workspace(
            "https://models.example.com".into(),
            "".into(),
            "".into(),
        ).await?;
        assert_eq!(entry.alias, "1");
        let answer = Workspace {
            id: 1,
            url: "https://models.example.com".into(),
            superceded_by_id: None,
            created_ts: 1234567890,
            description: Some("".into()),
            long_description: Some("".into()),
            exposures: None,
        };
        assert_eq!(entry.entity.into_inner(), answer);
        Ok(())
    }
}
