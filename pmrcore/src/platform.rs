use async_trait::async_trait;
use std::error::Error;

mod access_control;
mod managed_content;
mod task_management;
pub use access_control::{DefaultACPlatform, ACPlatform};
pub use managed_content::{DefaultMCPlatform, MCPlatform};
pub use task_management::{DefaultTMPlatform, TMPlatform};

pub trait PlatformUrl {
    fn url(&self) -> &str;
}

#[async_trait]
pub trait PlatformBuilder {
    async fn ac(url: impl ToString + Send) -> Result<impl ACPlatform, Box<dyn Error + Send + Sync + 'static>>;
    async fn mc(url: impl ToString + Send) -> Result<impl MCPlatform, Box<dyn Error + Send + Sync + 'static>>;
    async fn tm(url: impl ToString + Send) -> Result<impl TMPlatform, Box<dyn Error + Send + Sync + 'static>>;
}
