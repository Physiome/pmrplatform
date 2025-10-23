use async_trait::async_trait;
use crate::{
    citation::{
        Citation,
        CitationResourceSet,
    },
    error::BackendError,
};

#[async_trait]
pub trait CitationBackend {
    async fn add_citation(
        &self,
        identifier: &str,
    ) -> Result<i64, BackendError>;

    async fn add_citation_link(
        &self,
        citation_id: i64,
        resource_path: &str,
    ) -> Result<(), BackendError>;

    /// returns the full listing of `Citation`
    async fn list_citations(
        &self,
    ) -> Result<Vec<Citation>, BackendError>;

    /// returns the resource string identifiers for the given citation identifier
    async fn list_citation_resources(
        &self,
        identifier: &str,
    ) -> Result<Vec<String>, BackendError>;

    async fn get_citation_resource_set(
        &self,
        identifier: &str,
    ) -> Result<CitationResourceSet, BackendError> {
        todo!()
    }
}
