use pmrcore::{
    index::{
        ResourceKindedTermsCache,
        CachedIndexBackend,
    },
    platform::FullPCPlatform,
};
use std::sync::Arc;
use test_pmr::core::MockPlatform;

#[async_std::test]
async fn create_from_mock() {
    let platform = Arc::new(MockPlatform::new());
    let mem = ResourceKindedTermsCache::new(platform.clone());
    let _dc = CachedIndexBackend::new(platform.clone());
    let _mem_dc = ResourceKindedTermsCache::new(mem.into());
}

#[async_std::test]
async fn create_from_dyn() {
    // Just the plain PCPlatform cannot provide this.
    // let platform: Arc<dyn PCPlatform> = Arc::new(MockPlatform::new());
    // Instead it will also need `IndexCoreDBCache` to ensure that the disk cache version can be built.
    let platform: Arc<dyn FullPCPlatform> = Arc::new(MockPlatform::new());
    let mem = ResourceKindedTermsCache::new(platform.clone());
    let _dc = CachedIndexBackend::new(platform.clone());
    let _mem_dc = ResourceKindedTermsCache::new(mem.into());
}
