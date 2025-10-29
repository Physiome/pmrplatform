use pmrcore::platform::{
    MCPlatform,
    PCPlatform,
    TMPlatform,
};
use pmrrepo::backend::Backend;
use std::{
    path::PathBuf,
    sync::Arc,
};

#[derive(Clone)]
pub struct Platform {
    pub ac_platform: pmrac::Platform,
    pub mc_platform: Arc<dyn MCPlatform>,
    pub pc_platform: Arc<dyn PCPlatform>,
    pub tm_platform: Arc<dyn TMPlatform>,
    pub(crate) data_root: PathBuf,
    pub(crate) repo_root: PathBuf,
    pub(crate) repo_backend: Backend,
}
