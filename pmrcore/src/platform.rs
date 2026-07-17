mod access_control;
mod connector;
mod managed_content;
mod processed_content;
mod task_management;
pub use access_control::{DefaultACPlatform, ACPlatform, RawACPlatform};
pub use connector::{ConnectorOption, PlatformConnector};
pub use managed_content::{DefaultMCPlatform, MCPlatform, RawMCPlatform};
pub use processed_content::{DefaultPCPlatform, FullPCPlatform, PCPlatform, RawPCPlatform};
pub use task_management::{DefaultTMPlatform, TMPlatform, RawTMPlatform};

pub trait PlatformCore {
    fn url(&self) -> &str;
    // fn backend(&self) -> &Self::Backend;
}

pub trait RawPlatform {
    type Backend;

    fn backend(&self) -> &Self::Backend;
}
