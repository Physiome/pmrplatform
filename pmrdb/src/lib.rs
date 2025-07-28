use pmrcore::platform::{ConnectorOption, PlatformConnector, ACPlatform, MCPlatform, TMPlatform};
#[cfg(feature = "sqlite")]
use pmrdb_sqlite::SqliteBackend;

pub struct Backend;

#[derive(Clone, Debug, PartialEq)]
pub struct Error(String);

#[derive(Debug)]
enum BackendKind {
    Sqlite,
}

mod display {
    use super::{BackendKind, Error};
    use std::fmt::{Display, Formatter, Result};

    impl Display for BackendKind {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Self::Sqlite => "sqlite".fmt(f),
            }
        }
    }

    impl Display for Error {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            self.0.fmt(f)
        }
    }

    impl std::error::Error for Error {}
}

impl TryFrom<&str> for BackendKind {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.split(':').next() {
            Some("sqlite") => Ok(BackendKind::Sqlite),
            _ => Err(Error(format!("The connection string {s:?} is unsupported.")))
        }
    }
}

impl Backend {
    pub async fn ac(
        opts: ConnectorOption
    ) -> Result<Box<dyn ACPlatform>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        match BackendKind::try_from(opts.url.as_str()) {
            #[cfg(feature = "sqlite")]
            Ok(BackendKind::Sqlite) => Ok(Box::new(SqliteBackend::ac(opts).await?)),
            #[cfg(not(feature = "sqlite"))]
            Ok(s) => Err(Box::new(Error(format!(
                "The feature {s:?} must be enabled for pmrdb in order to connect to {:?}", opts.url
            )))),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn mc(
        opts: ConnectorOption
    ) -> Result<Box<dyn MCPlatform>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        match BackendKind::try_from(opts.url.as_str()) {
            #[cfg(feature = "sqlite")]
            Ok(BackendKind::Sqlite) => Ok(Box::new(SqliteBackend::mc(opts).await?)),
            #[cfg(not(feature = "sqlite"))]
            Ok(s) => Err(Box::new(Error(format!(
                "The feature {s:?} must be enabled for pmrdb in order to connect to {:?}", opts.url
            )))),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn tm(
        opts: ConnectorOption
    ) -> Result<Box<dyn TMPlatform>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        match BackendKind::try_from(opts.url.as_str()) {
            #[cfg(feature = "sqlite")]
            Ok(BackendKind::Sqlite) => Ok(Box::new(SqliteBackend::tm(opts).await?)),
            #[cfg(not(feature = "sqlite"))]
            Ok(s) => Err(Box::new(Error(format!(
                "The feature {s:?} must be enabled for pmrdb in order to connect to {:?}", opts.url
            )))),
            Err(e) => Err(Box::new(e)),
        }
    }
}

#[cfg(test)]
mod testing {
    use crate::Backend;

    #[async_std::test]
    async fn smoke() {
        // simple round-trip testing
        assert!(Backend::mc("unsupported".into()).await.is_err());
    }

    #[cfg(feature = "sqlite")]
    #[async_std::test]
    async fn smoke_sqlite() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // simple round-trip testing
        let mcp = Backend::mc("sqlite::memory:".into()).await?;
        let workspace_id = mcp.add_workspace("title", "description", "").await?;
        assert_eq!(mcp.get_workspace(workspace_id).await?.into_inner().id, workspace_id);
        Ok(())
    }
}
