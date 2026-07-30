use pmrcore::{
    index::traits::IndexBackend,
    platform::{
        MCPlatform,
        PCPlatform,
        TMPlatform,
    },
};
use pmrrepo::backend::Backend;
use std::{
    path::PathBuf,
    sync::Arc,
};

#[derive(Clone)]
pub struct Platform {
    pub(crate) ac_platform: pmrac::Platform,
    pub(crate) mc_platform: Arc<dyn MCPlatform>,
    pub(crate) pc_platform: Arc<dyn PCPlatform>,
    pub(crate) tm_platform: Arc<dyn TMPlatform>,
    pub(crate) index_backend: Arc<dyn IndexBackend>,
    pub(crate) data_root: PathBuf,
    pub(crate) repo_root: PathBuf,
    pub(crate) repo_backend: Backend,
}
