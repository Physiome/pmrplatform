pub mod error;
pub mod password;
pub mod platform;
pub mod session;
pub mod user;

pub type Platform = std::sync::Arc<platform::Platform>;
