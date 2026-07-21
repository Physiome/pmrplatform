use async_trait::async_trait;
use std::collections::HashSet;
use crate::error::BackendError;
use super::*;

#[async_trait]
pub trait IndexCoreDBBackend {
    /// This resolves the `id` associated with `kind`; if not already exist it will be created and
    /// its `id` returned.
    async fn resolve_kind(
        &self,
        kind: &str,
    ) -> Result<i64, BackendError>;
    /// This resolves the `id` associated with `idx_kind_id` and `term`; if not already exist it will
    /// be created and its `id` returned.
    async fn resolve_idx_entry(
        &self,
        idx_kind_id: i64,
        term: &str,
    ) -> Result<i64, BackendError>;
    /// Link the `resource_path` to the `idx_entry_id` associated with the pair of index kind and the
    /// term.
    async fn add_idx_entry_link(
        &self,
        idx_entry_id: i64,
        resource_path: &str,
    ) -> Result<(), BackendError>;
    /// Link `resource_path` with the text content for the text index.
    async fn add_idx_text_core(
        &self,
        title: Option<&str>,
        content: Option<&str>,
        resource_path: &str,
    ) -> Result<(), BackendError>;
    /// Forget the `resource_path` from the index.
    async fn forget_resource_path_core(
        &self,
        kind: Option<&str>,
        resource_path: &str,
    ) -> Result<(), BackendError>;
    /// Forget the text associated with `resource_path`.
    async fn forget_resource_text_core(
        &self,
        resource_path: &str,
    ) -> Result<(), BackendError>;

    /// List the kinds of indexes available.
    async fn list_kinds_core(&self) -> Result<Vec<String>, BackendError>;
    /// List the terms available under the kind
    async fn list_terms_core(
        &self,
        kind: &str,
    ) -> Result<Option<IndexTerms>, BackendError>;
    /// List the resources available under the kind
    ///
    /// A `Ok(None)` result should mean the kind is unknown.
    async fn list_resources_core(
        &self,
        kind: &str,
        term: &str,
    ) -> Result<Option<IndexResourceSet>, BackendError>;
    /// List the text associated with the resources given the provided text.
    ///
    /// An optional bracket may be provided to highlight the matched text.
    async fn list_resources_text_core(
        &self,
        text: &str,
        bracket: Option<(&str, &str)>,
    ) -> Result<Vec<ResourceBrief>, BackendError>;

    /// Get the kinded terms for the given resource path
    async fn get_resource_kinded_terms_core(
        &self,
        resource_path: &str,
    ) -> Result<ResourceKindedTerms, BackendError>;

    /// Get the brief associated with the resource path.
    async fn get_resource_brief_core(
        &self,
        resource_path: &str,
    ) -> Result<Option<ResourceBrief>, BackendError>;
}

#[async_trait]
pub trait IndexCoreDBCache {
    // TODO
    // - Provide an in-memory wrapper instead, or in conjunction with this denormalized,
    //   be part of a concrete wrapper that implements subparts of this trait
    /// Cache the kinded terms for the given resource path at the database before returning it.
    async fn cache_resource_kinded_terms(
        &self,
        resource_path: &str,
    ) -> Result<ResourceKindedTerms, BackendError>;

    /// Get the kinded terms for the given resource path from the cache.
    ///
    /// `None` will be returned the database failed to produce the value.
    async fn get_cached_resource_kinded_terms(
        &self,
        resource_path: &str,
    ) -> Result<Option<ResourceKindedTerms>, BackendError>;

    /// Uncache the kinded terms for the given resource path.
    async fn uncache_resource_kinded_terms(
        &self,
        resource_path: &str,
    ) -> Result<(), BackendError>;

    /// Uncache all kinded terms.
    async fn uncache_all_resource_kinded_terms(
        &self,
    ) -> Result<(), BackendError>;
}

// TODO not make this a super trait but have this implement only for `IndexBackend`.
// idea is to have the cache layer implement this.
#[async_trait]
pub trait IndexCoreBackend {
    async fn add_idx_text(
        &self,
        title: Option<&str>,
        content: Option<&str>,
        resource_path: &str,
    ) -> Result<(), BackendError>;

    async fn forget_resource_path(
        &self,
        kind: Option<&str>,
        resource_path: &str,
    ) -> Result<(), BackendError>;

    async fn forget_resource_text(
        &self,
        resource_path: &str,
    ) -> Result<(), BackendError>;

    /// List the kinds of indexes available.
    async fn list_kinds(&self) -> Result<Vec<String>, BackendError>;

    /// List the terms available under the kind
    async fn list_terms(
        &self,
        kind: &str,
    ) -> Result<Option<IndexTerms>, BackendError>;

    /// List the resources available under the kind
    ///
    /// A `Ok(None)` result should mean the kind is unknown.
    async fn list_resources(
        &self,
        kind: &str,
        term: &str,
    ) -> Result<Option<IndexResourceSet>, BackendError>;

    /// List the text associated with the resources given the provided text.
    ///
    /// An optional bracket may be provided to highlight the matched text.
    async fn list_resources_text(
        &self,
        text: &str,
        bracket: Option<(&str, &str)>,
    ) -> Result<Vec<ResourceBrief>, BackendError>;

    /// Get the kinded terms for the given resource path.
    async fn get_resource_kinded_terms(
        &self,
        resource_path: &str,
    ) -> Result<ResourceKindedTerms, BackendError>;

    /// Get the brief for the given resource path.
    async fn get_resource_brief(
        &self,
        resource_path: &str,
    ) -> Result<Option<ResourceBrief>, BackendError>;
}

#[async_trait]
impl<T> IndexCoreBackend for T
where
    T: IndexCoreDBBackend + Sync
{
    async fn add_idx_text(
        &self,
        title: Option<&str>,
        content: Option<&str>,
        resource_path: &str,
    ) -> Result<(), BackendError> {
        self.add_idx_text_core(title, content, resource_path).await
    }

    async fn forget_resource_path(
        &self,
        kind: Option<&str>,
        resource_path: &str,
    ) -> Result<(), BackendError> {
        self.forget_resource_path_core(kind, resource_path).await
    }

    async fn forget_resource_text(
        &self,
        resource_path: &str,
    ) -> Result<(), BackendError> {
        self.forget_resource_text_core(resource_path).await
    }

    /// List the kinds of indexes available.
    async fn list_kinds(&self) -> Result<Vec<String>, BackendError> {
        self.list_kinds_core().await
    }

    /// List the terms available under the kind
    async fn list_terms(
        &self,
        kind: &str,
    ) -> Result<Option<IndexTerms>, BackendError> {
        self.list_terms_core(kind).await
    }

    /// List the resources available under the kind
    ///
    /// A `Ok(None)` result should mean the kind is unknown.
    async fn list_resources(
        &self,
        kind: &str,
        term: &str,
    ) -> Result<Option<IndexResourceSet>, BackendError> {
        self.list_resources_core(kind, term).await
    }

    /// List the text associated with the resources given the provided text.
    ///
    /// An optional bracket may be provided to highlight the matched text.
    async fn list_resources_text(
        &self,
        text: &str,
        bracket: Option<(&str, &str)>,
    ) -> Result<Vec<ResourceBrief>, BackendError> {
        self.list_resources_text_core(text, bracket).await
    }

    /// Get the kinded terms for the given resource path.
    async fn get_resource_kinded_terms(
        &self,
        resource_path: &str,
    ) -> Result<ResourceKindedTerms, BackendError> {
        self.get_resource_kinded_terms_core(resource_path).await
    }

    /// Get the brief for the given resource path.
    async fn get_resource_brief(
        &self,
        resource_path: &str,
    ) -> Result<Option<ResourceBrief>, BackendError> {
        self.get_resource_brief_core(resource_path).await
    }
}

#[async_trait]
pub trait IndexBackend: IndexCoreBackend + Send + Sync {
    async fn resource_link_kind_with_terms(
        &self,
        resource_path: &str,
        kind: &str,
        terms: &mut (dyn Iterator<Item = &str> + Send + Sync),
    ) -> Result<(), BackendError>;

    async fn resource_link_kind_with_term(
        &self,
        resource_path: &str,
        kind: &str,
        term: &str,
    ) -> Result<(), BackendError>;

    async fn get_resource_details(
        &self,
        resource_path: &str,
    ) -> Result<ResourceKindedTerms, BackendError> {
        let mut kinded_terms = self.get_resource_kinded_terms(resource_path).await?;
        let brief = self.get_resource_brief(resource_path).await?.unwrap_or_default();
        kinded_terms.data.insert(String::from("_title"), brief.title.into_iter().collect());
        kinded_terms.data.insert(String::from("_brief"), brief.brief.into_iter().collect());
        Ok(kinded_terms)
    }

    /// Get the kinded terms for the given resource path
    async fn list_resources_details(
        &self,
        kind: &str,
        term: &str,
    ) -> Result<Option<IndexResourceDetailedSet>, BackendError> {
        if let Some(IndexResourceSet {
            kind,
            term,
            resource_paths,
        }) = self.list_resources(kind, term).await? {
            let mut results = Vec::new();
            for resource_path in resource_paths.into_iter() {
                results.push(self.get_resource_details(&resource_path).await?);
            }
            Ok(Some(IndexResourceDetailedSet {
                kind,
                term,
                resource_paths: results
            }))
        } else {
            Ok(None)
        }
    }

    /// Convert a resource brief into resource kinded terms, with the `title` and `brief` field encoded
    /// as keys prefixed with `_` inside data.
    async fn resource_brief_to_kinded_terms(
        &self,
        brief: ResourceBrief,
    ) -> Result<ResourceKindedTerms, BackendError> {
        let mut kinded_terms = self.get_resource_kinded_terms(&brief.resource_path).await?;
        kinded_terms.data.insert(String::from("_title"), brief.title.into_iter().collect());
        kinded_terms.data.insert(String::from("_brief"), brief.brief.into_iter().collect());
        Ok(kinded_terms)
    }

    /// List the text associated with the resources given the provided text, but converted to a
    /// `ResourceKindedTerms` using `resource_brief_to_kinded_terms`.
    ///
    /// An optional bracket may be provided to highlight the matched text.
    async fn list_resources_text_as_kinded_terms(
        &self,
        text: &str,
        bracket: Option<(&str, &str)>,
    ) -> Result<Vec<ResourceKindedTerms>, BackendError> {
        let mut results = Vec::new();
        for brief in self.list_resources_text(text, bracket)
            .await?
            .into_iter()
        {
            results.push(self.resource_brief_to_kinded_terms(brief).await?);
        }
        Ok(results)
    }

    async fn query_resource(
        &self,
        Query { query, filters }: &Query,
        bracket: Option<(&str, &str)>,
    ) -> Result<Vec<ResourceKindedTerms>, BackendError> {
        // 1. If there is a query, gather the briefs to be used later.
        let briefs = if let Some(query) = query {
            Some(self.list_resources_text(query, bracket).await?)
        } else {
            None
        };

        // 2. If filters are empty, simply return the briefs as is.
        if filters.is_empty() {
            let mut results = Vec::new();
            for brief in briefs.unwrap_or_default().into_iter() {
                results.push(self.resource_brief_to_kinded_terms(brief).await?);
            }
            return Ok(results);
        }

        // 3.1 Use the first filter, and set up the initial set of resource paths.
        let mut filters = filters.into_iter();
        let mut resource_paths: HashSet<String> = if let Some(Filter { kind, term }) = filters.next() {
            self.list_resources(kind, term).await?
                .unwrap_or_default()
                .resource_paths
                .into_iter()
                .collect()
        } else {
            Default::default()
        };

        // 3.2 Intersect every remaining results with kind/term pairs (i.e. `and` everything together).
        for Filter { kind, term } in filters.into_iter() {
            resource_paths = resource_paths
                .intersection(
                    &self.list_resources(kind, term).await?
                        .unwrap_or_default()
                        .resource_paths
                        .into_iter()
                        .collect::<HashSet<String>>()
                )
                .cloned()
                .collect();
        }

        let mut results = Vec::new();
        if let Some(briefs) = briefs {
            // 4.1 Briefs will be available with text query, so combine that with the resources.
            for brief in briefs.into_iter() {
                if resource_paths.contains(&brief.resource_path) {
                    results.push(self.resource_brief_to_kinded_terms(brief).await?);
                }
            }
        } else {
            // 4.2. No text query so no briefs, so simply get everything.
            for resource_path in resource_paths.into_iter() {
                results.push(self.get_resource_details(&resource_path).await?);
            }
        }
        Ok(results)
    }
}

#[async_trait]
impl<T> IndexBackend for T
where
    T: IndexCoreDBBackend + Send + Sync
{
    async fn resource_link_kind_with_terms(
        &self,
        resource_path: &str,
        kind: &str,
        terms: &mut (dyn Iterator<Item = &str> + Send + Sync),
    ) -> Result<(), BackendError> {
        let idx_kind_id = self.resolve_kind(kind).await?;

        for term in terms {
            let idx_entry_id = self.resolve_idx_entry(idx_kind_id, term).await?;
            self.add_idx_entry_link(idx_entry_id, &resource_path).await?;
        }
        Ok(())
    }

    async fn resource_link_kind_with_term(
        &self,
        resource_path: &str,
        kind: &str,
        term: &str,
    ) -> Result<(), BackendError> {
        let idx_kind_id = self.resolve_kind(kind).await?;
        let idx_entry_id = self.resolve_idx_entry(idx_kind_id, term).await?;
        self.add_idx_entry_link(idx_entry_id, &resource_path).await?;
        Ok(())
    }
}
