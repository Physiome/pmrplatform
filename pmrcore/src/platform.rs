mod access_control;
mod connector;
mod managed_content;
mod processed_content;
mod task_management;
pub use access_control::{DefaultACPlatform, ACPlatform};
pub use connector::{ConnectorOption, PlatformConnector};
pub use managed_content::{DefaultMCPlatform, MCPlatform};
pub use processed_content::{DefaultPCPlatform, PCPlatform};
pub use task_management::{DefaultTMPlatform, TMPlatform};

pub trait PlatformUrl {
    fn url(&self) -> &str;
}
