use pmrcore::platform::{
    MCPlatform,
    TMPlatform,
};
use pmrrepo::backend::Backend;
use std::{
    path::PathBuf,
    sync::Arc,
};

#[derive(Clone)]
pub struct Platform {
    pub mc_platform: Arc<dyn MCPlatform + Send + Sync>,
    pub tm_platform: Arc<dyn TMPlatform + Send + Sync>,
    pub(crate) data_root: PathBuf,
    pub(crate) repo_root: PathBuf,
    pub(crate) repo_backend: Backend,
}
