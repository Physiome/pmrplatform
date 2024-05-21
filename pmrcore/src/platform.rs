mod managed_content;
mod task_management;
pub use managed_content::MCPlatform;
pub use task_management::TMPlatform;

pub trait PlatformUrl {
    fn url(&self) -> &str;
}
