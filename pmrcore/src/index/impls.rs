use async_trait::async_trait;
use std::{
    collections::BTreeMap,
    sync::Arc,
};

use crate::error::BackendError;
use super::{
    CachedIndexBackend,
    IndexResourceSet,
    IndexTerms,
    ResourceBrief,
    ResourceKindedTerms,
    ResourceKindedTermsCache,
    traits::{IndexBackend, IndexCoreDBCache, IndexCoreBackend},
};

impl<B> ResourceKindedTermsCache<B> {
    fn insert(&self, resource_path: &str, data: &BTreeMap<String, Vec<String>>) {
        if let Ok(mut heap) = self.heap.write() {
            heap.insert(resource_path.to_owned(), data.to_owned());
        } else {
            // Log the poisoned lock?
        }
    }

    fn update(&self, resource_path: &str, kind: &str, terms: &[String]) {
        if let Ok(mut heap) = self.heap.write() {
            heap.entry(resource_path.to_string())
                .and_modify(|data| {
                    data.entry(kind.to_string())
                        .and_modify(|existing_terms| existing_terms.extend_from_slice(terms))
                        .or_insert_with(|| terms.to_vec());
                })
                .or_insert_with(|| [(kind.to_owned(), terms.to_owned())].into());
        } else {
            // Log the poisoned lock?
        }
    }

    fn cache(
        &self,
        ResourceKindedTerms { resource_path, data }: &ResourceKindedTerms,
    ) {
        self.insert(resource_path, data)
    }

    fn remove(&self, resource_path: &str) {
        if let Ok(mut heap) = self.heap.write() {
            heap.remove(resource_path);
        } else {
            // Log the poisoned lock?
        }
    }

    fn get(&self, resource_path: &str) -> Option<ResourceKindedTerms> {
        if let Ok(heap) = self.heap.read() {
            if let Some(data) = heap.get(resource_path) {
                return Some(ResourceKindedTerms {
                    resource_path: resource_path.to_owned(),
                    data: data.to_owned(),
                });
            }
        } else {
            // Log the poisoned lock?
        }
        None
    }
}

impl<B> ResourceKindedTermsCache<B>
where
    B: IndexCoreBackend + Send + Sync,
{
    pub fn new(backend: Arc<B>) -> Self {
        Self {
            backend,
            heap: Default::default(),
        }
    }
}

#[async_trait]
impl<B> IndexCoreBackend for ResourceKindedTermsCache<B>
where
    B: IndexCoreBackend + Send + Sync,
{
    async fn add_idx_text(
        &self,
        title: Option<&str>,
        content: Option<&str>,
        resource_path: &str,
    ) -> Result<(), BackendError> {
        self.backend.add_idx_text(title, content, resource_path).await
    }

    async fn forget_resource_path(
        &self,
        kind: Option<&str>,
        resource_path: &str,
    ) -> Result<(), BackendError> {
        self.backend.forget_resource_path(kind, resource_path).await?;
        self.remove(resource_path);
        Ok(())
    }

    async fn forget_resource_text(
        &self,
        resource_path: &str,
    ) -> Result<(), BackendError> {
        self.backend.forget_resource_text(resource_path).await
    }

    /// List the kinds of indexes available.
    async fn list_kinds(&self) -> Result<Vec<String>, BackendError> {
        self.backend.list_kinds().await
    }

    /// List the terms available under the kind
    async fn list_terms(
        &self,
        kind: &str,
    ) -> Result<Option<IndexTerms>, BackendError> {
        self.backend.list_terms(kind).await
    }

    /// List the resources available under the kind
    ///
    /// A `Ok(None)` result should mean the kind is unknown.
    async fn list_resources(
        &self,
        kind: &str,
        term: &str,
    ) -> Result<Option<IndexResourceSet>, BackendError> {
        self.backend.list_resources(kind, term).await
    }

    /// List the text associated with the resources given the provided text.
    ///
    /// An optional bracket may be provided to highlight the matched text.
    async fn list_resources_text(
        &self,
        text: &str,
        bracket: Option<(&str, &str)>,
    ) -> Result<Vec<ResourceBrief>, BackendError> {
        self.backend.list_resources_text(text, bracket).await
    }

    /// Get the kinded terms for the given resource path.
    async fn get_resource_kinded_terms(
        &self,
        resource_path: &str,
    ) -> Result<ResourceKindedTerms, BackendError> {
        if let Some(cached_value) = self.get(resource_path) {
            return Ok(cached_value);
        }
        let result = self.backend.get_resource_kinded_terms(resource_path).await?;
        self.cache(&result);
        Ok(result)
    }

    /// Get the brief for the given resource path.
    async fn get_resource_brief(
        &self,
        resource_path: &str,
    ) -> Result<Option<ResourceBrief>, BackendError> {
        self.backend.get_resource_brief(resource_path).await
    }
}

#[async_trait]
impl<B> IndexBackend for ResourceKindedTermsCache<B>
where
    B: IndexBackend + Send + Sync,
{
    async fn resource_link_kind_with_terms(
        &self,
        resource_path: &str,
        kind: &str,
        terms: &mut (dyn Iterator<Item = &str> + Send + Sync),
    ) -> Result<(), BackendError> {
        let terms = terms.collect::<Vec<_>>();
        self.backend.resource_link_kind_with_terms(resource_path, kind, &mut terms.iter().map(|s| *s)).await?;
        self.update(resource_path, kind, &terms.into_iter().map(String::from).collect::<Vec<_>>());
        Ok(())
    }

    async fn resource_link_kind_with_term(
        &self,
        resource_path: &str,
        kind: &str,
        term: &str,
    ) -> Result<(), BackendError> {
        self.backend.resource_link_kind_with_term(resource_path, kind, term).await?;
        self.update(resource_path, kind, &[String::from(term)]);
        Ok(())
    }
}

impl<B> CachedIndexBackend<B>
where
    B: IndexCoreDBCache + IndexCoreBackend + Send + Sync,
{
    pub fn new(backend: Arc<B>) -> Self {
        Self {
            backend,
        }
    }
}

#[async_trait]
impl<B> IndexCoreBackend for CachedIndexBackend<B>
where
    B: IndexCoreDBCache + IndexCoreBackend + Send + Sync,
{
    async fn add_idx_text(
        &self,
        title: Option<&str>,
        content: Option<&str>,
        resource_path: &str,
    ) -> Result<(), BackendError> {
        self.backend.add_idx_text(title, content, resource_path).await
    }

    async fn forget_resource_path(
        &self,
        kind: Option<&str>,
        resource_path: &str,
    ) -> Result<(), BackendError> {
        self.backend.forget_resource_path(kind, resource_path).await?;
        // TODO dedicate error for cache failing to uncache?
        self.backend.uncache_resource_kinded_terms(resource_path).await?;
        Ok(())
    }

    async fn forget_resource_text(
        &self,
        resource_path: &str,
    ) -> Result<(), BackendError> {
        self.backend.forget_resource_text(resource_path).await
    }

    /// List the kinds of indexes available.
    async fn list_kinds(&self) -> Result<Vec<String>, BackendError> {
        self.backend.list_kinds().await
    }

    /// List the terms available under the kind
    async fn list_terms(
        &self,
        kind: &str,
    ) -> Result<Option<IndexTerms>, BackendError> {
        self.backend.list_terms(kind).await
    }

    /// List the resources available under the kind
    ///
    /// A `Ok(None)` result should mean the kind is unknown.
    async fn list_resources(
        &self,
        kind: &str,
        term: &str,
    ) -> Result<Option<IndexResourceSet>, BackendError> {
        self.backend.list_resources(kind, term).await
    }

    /// List the text associated with the resources given the provided text.
    ///
    /// An optional bracket may be provided to highlight the matched text.
    async fn list_resources_text(
        &self,
        text: &str,
        bracket: Option<(&str, &str)>,
    ) -> Result<Vec<ResourceBrief>, BackendError> {
        self.backend.list_resources_text(text, bracket).await
    }

    /// Get the kinded terms for the given resource path.
    async fn get_resource_kinded_terms(
        &self,
        resource_path: &str,
    ) -> Result<ResourceKindedTerms, BackendError> {
        if let Some(cached_value) = self.backend.get_cached_resource_kinded_terms(resource_path).await? {
            return Ok(cached_value);
        }
        // Simply invoke the cache method directly, which should return whatever that got just cached.
        self.backend.cache_resource_kinded_terms(resource_path).await
    }

    /// Get the brief for the given resource path.
    async fn get_resource_brief(
        &self,
        resource_path: &str,
    ) -> Result<Option<ResourceBrief>, BackendError> {
        self.backend.get_resource_brief(resource_path).await
    }
}

#[async_trait]
impl<B> IndexBackend for CachedIndexBackend<B>
where
    B: IndexCoreDBCache + IndexBackend + Send + Sync,
{
    async fn resource_link_kind_with_terms(
        &self,
        resource_path: &str,
        kind: &str,
        terms: &mut (dyn Iterator<Item = &str> + Send + Sync),
    ) -> Result<(), BackendError> {
        self.backend.resource_link_kind_with_terms(resource_path, kind, terms).await?;
        // Rather than uncache, pre-cache it.
        // self.backend.uncache_resource_kinded_terms(resource_path).await?;
        self.backend.cache_resource_kinded_terms(resource_path).await?;
        Ok(())
    }

    async fn resource_link_kind_with_term(
        &self,
        resource_path: &str,
        kind: &str,
        term: &str,
    ) -> Result<(), BackendError> {
        self.backend.resource_link_kind_with_term(resource_path, kind, term).await?;
        // Rather than uncache, pre-cache it.
        // self.backend.uncache_resource_kinded_terms(resource_path).await?;
        self.backend.cache_resource_kinded_terms(resource_path).await?;
        Ok(())
    }
}
