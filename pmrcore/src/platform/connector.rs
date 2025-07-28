use async_trait::async_trait;
use std::error::Error;

use super::*;

#[derive(Default)]
pub struct ConnectorOption {
    pub create_db: bool,
    pub url: String,
}

impl ConnectorOption {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_db(mut self, create_db: bool) -> Self {
        self.create_db = create_db;
        self
    }

    pub fn url(mut self, url: String) -> Self {
        self.url = url;
        self
    }
}

impl<T> From<T> for ConnectorOption
where
    T: ToString
{
    fn from(v: T) -> Self {
        Self::new().url(v.to_string())
    }
}

#[async_trait]
pub trait PlatformConnector {
    async fn ac(opts: ConnectorOption) -> Result<impl ACPlatform, Box<dyn Error + Send + Sync + 'static>>;
    async fn mc(opts: ConnectorOption) -> Result<impl MCPlatform, Box<dyn Error + Send + Sync + 'static>>;
    async fn tm(opts: ConnectorOption) -> Result<impl TMPlatform, Box<dyn Error + Send + Sync + 'static>>;
}
