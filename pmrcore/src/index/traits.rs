use async_trait::async_trait;
use crate::error::BackendError;
use super::*;

#[async_trait]
pub trait IndexBackend {
    /// This resolves the `id` associated with `kind`; if not already exist it will be created and
    /// its `id` returned.
    async fn resolve_kind(
        &self,
        kind: impl AsRef<str> + Send + Sync,
    ) -> Result<i64, BackendError>;
    /// This resolves the `id` associated with `idx_kind_id` and `term`; if not already exist it will
    /// be created and its `id` returned.
    async fn resolve_idx_entry(
        &self,
        idx_kind_id: i64,
        term: impl AsRef<str> + Send + Sync,
    ) -> Result<i64, BackendError>;
    /// Link the `resource_path` to the `idx_entry_id` associated with the pair of index kind and the
    /// term.
    async fn add_idx_entry_link(
        &self,
        idx_entry_id: i64,
        resource_path: impl AsRef<str> + Send + Sync,
    ) -> Result<(), BackendError>;
    /// Forget the `resource_path` from the index.
    async fn forget_resource_path(
        &self,
        kind: Option<impl AsRef<str> + Send + Sync>,
        resource_path: impl AsRef<str> + Send + Sync,
    ) -> Result<(), BackendError>;

    /// List the kinds of indexes available.
    async fn list_kinds(&self) -> Result<Vec<String>, BackendError>;
    /// List the terms available under the kind
    async fn list_terms(
        &self,
        kind: impl AsRef<str> + Send + Sync,
    ) -> Result<IndexTerms, BackendError>;
    /// List the resources available under the kind
    async fn list_resources(
        &self,
        kind: impl AsRef<str> + Send + Sync,
        term: impl AsRef<str> + Send + Sync,
    ) -> Result<IndexResourceSet, BackendError>;

    async fn index_resource(
        &self,
        kind: impl AsRef<str> + Send + Sync,
        resource_path: impl AsRef<str> + Send + Sync,
        terms: &[impl AsRef<str> + Send + Sync],
    ) -> Result<(), BackendError> {
        let idx_kind_id = self.resolve_kind(kind).await?;

        for term in terms.iter() {
            let idx_entry_id = self.resolve_idx_entry(idx_kind_id, term).await?;
            self.add_idx_entry_link(idx_entry_id, &resource_path).await?;
        }
        Ok(())
    }
}
