mod access_control;
mod managed_content;
mod task_management;
pub use access_control::ACPlatform;
pub use managed_content::MCPlatform;
pub use task_management::TMPlatform;

pub trait PlatformUrl {
    fn url(&self) -> &str;
}
