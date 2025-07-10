mod access_control;
mod managed_content;
mod task_management;
pub use access_control::{DefaultACPlatform, ACPlatform};
pub use managed_content::{DefaultMCPlatform, MCPlatform};
pub use task_management::{DefaultTMPlatform, TMPlatform};

pub trait PlatformUrl {
    fn url(&self) -> &str;
}
