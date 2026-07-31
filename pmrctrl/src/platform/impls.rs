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
    fmt,
    path::{
        Path,
        PathBuf,
    },
    sync::Arc,
};

use crate::{
    platform::types::{
        Platform,
        PlatformInner,
    },
};

impl Platform {
    pub(crate) fn new(
        ac_platform: pmrac::Platform,
        mc_platform: Arc<dyn MCPlatform>,
        pc_platform: Arc<dyn PCPlatform>,
        tm_platform: Arc<dyn TMPlatform>,
        index_backend: Arc<dyn IndexBackend>,
        data_root: PathBuf,
        repo_root: PathBuf,
    ) -> Self {
        let repo_backend = Backend::new(mc_platform.clone(), repo_root.clone());
        Self {
            inner: Arc::new(PlatformInner {
                ac_platform,
                mc_platform,
                pc_platform,
                tm_platform,
                index_backend,
                data_root,
                repo_root,
                repo_backend,
            }),
            context: Default::default(),
        }
    }

    pub fn ac_platform(&self) -> &pmrac::Platform {
        &self.inner.ac_platform
    }

    pub fn ac_platform_clone(&self) -> pmrac::Platform {
        self.inner.ac_platform.clone()
    }

    pub fn mc_platform(&self) -> &dyn MCPlatform {
        self.inner.mc_platform.as_ref()
    }

    pub fn pc_platform(&self) -> &dyn PCPlatform {
        self.inner.pc_platform.as_ref()
    }

    pub fn tm_platform(&self) -> &dyn TMPlatform {
        self.inner.tm_platform.as_ref()
    }

    pub fn index_backend(&self) -> &dyn IndexBackend {
        self.inner.index_backend.as_ref()
    }

    pub fn data_root(&self) -> &Path {
        self.inner.data_root.as_ref()
    }

    pub fn repo_root(&self) -> &Path {
        self.inner.repo_root.as_ref()
    }

    pub fn repo_backend(&self) -> &Backend {
        &self.inner.repo_backend
    }

    // TODO see if we need to wholesale extend contexts to be efficient, but given how we likely only
    // have one or two with aggregated fields, this may be sufficient for now.
    /// Return a cloned platform with an additional context to the clone.
    pub fn add_context<T>(self, val: T) -> Self
    where
        T: Send + Sync + 'static,
    {
        let inner = self.inner.clone();
        let mut context = Arc::unwrap_or_clone(self.context);
        context.insert(val);
        let context = Arc::new(context);
        Self { inner, context }
    }

    pub fn use_context<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.context.get()
    }
}

impl fmt::Debug for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Platform")
            .field("data_root", &self.inner.data_root)
            .field("repo_root", &self.inner.repo_root)
            .finish()
    }
}

mod ac;
mod alias;
mod exposure;
mod profile;
mod task;
mod workspace;
