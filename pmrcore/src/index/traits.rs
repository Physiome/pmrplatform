use async_trait::async_trait;
use crate::error::BackendError;
use super::*;

#[async_trait]
pub trait IndexBackend {
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
    /// Forget the `resource_path` from the index.
    async fn forget_resource_path(
        &self,
        kind: Option<&str>,
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

    /// Get the kinded terms for the given resource path
    async fn get_resource_kinded_terms(
        &self,
        resource_path: &str,
    ) -> Result<ResourceKindedTerms, BackendError>;

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
                results.push(self.get_resource_kinded_terms(&resource_path).await?);
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
}
