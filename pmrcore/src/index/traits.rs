use async_trait::async_trait;
use std::collections::HashSet;
use crate::error::BackendError;
use super::*;

#[async_trait]
pub trait IndexCoreBackend {
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
    async fn add_idx_text(
        &self,
        title: Option<&str>,
        content: Option<&str>,
        resource_path: &str,
    ) -> Result<(), BackendError>;
    /// Forget the `resource_path` from the index.
    async fn forget_resource_path(
        &self,
        kind: Option<&str>,
        resource_path: &str,
    ) -> Result<(), BackendError>;
    /// Forget the text associated with `resource_path`.
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

    /// Get the kinded terms for the given resource path
    async fn get_resource_kinded_terms(
        &self,
        resource_path: &str,
    ) -> Result<ResourceKindedTerms, BackendError>;

    /// Get the brief associated with the resource path.
    async fn get_resource_brief(
        &self,
        resource_path: &str,
    ) -> Result<Option<ResourceBrief>, BackendError>;
}

#[async_trait]
pub trait IndexCoreCache {
    // TODO Revisit this design later, perhaps consider:
    // - Provide an in-memory wrapper instead, or in conjunction with this denormalized
    // - This be part of a concrete wrapper that implements subparts of this trait
    // - Rather, this trait be broken up into separate portions with the lower level core containing
    //   only the database access, while the pre-implemented part be a supertrait, with further custom
    //   implementation for the wrapper.
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
}

#[async_trait]
pub trait IndexBackend: IndexCoreBackend {
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

impl<T> IndexBackend for T
where
    T: IndexCoreBackend
{
}
