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
pub struct PlatformInner {
    pub(super) ac_platform: pmrac::Platform,
    pub(super) mc_platform: Arc<dyn MCPlatform>,
    pub(super) pc_platform: Arc<dyn PCPlatform>,
    pub(super) tm_platform: Arc<dyn TMPlatform>,
    pub(super) index_backend: Arc<dyn IndexBackend>,
    pub(super) data_root: PathBuf,
    pub(super) repo_root: PathBuf,
    pub(super) repo_backend: Backend,
}

#[derive(Clone)]
pub struct Platform {
    pub(super) inner: Arc<PlatformInner>,
    // pub(super) context: (),
}
