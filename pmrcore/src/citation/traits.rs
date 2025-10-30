use async_trait::async_trait;
use crate::{
    citation::{
        Citation,
        CitationResourceSet,
    },
    error::{
        BackendError,
        Error,
        ValueError,
    },
};

#[async_trait]
pub trait CitationBackend {
    async fn add_citation(
        &self,
        identifier: &str,
    ) -> Result<i64, BackendError>;

    /// Get a particular citation
    async fn get_citation_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<Citation>, BackendError>;

    /// returns the full listing of `Citation`
    async fn list_citations(
        &self,
    ) -> Result<Vec<Citation>, BackendError>;

    async fn add_citation_link(
        &self,
        citation_id: i64,
        resource_path: &str,
    ) -> Result<(), BackendError>;

    /// returns the resource string identifiers for the given citation identifier
    async fn list_citation_resources(
        &self,
        identifier: &str,
    ) -> Result<Vec<String>, BackendError>;

    async fn link_citation(
        &self,
        identifier: &str,
        resource_path: &str,
    ) -> Result<(), Error> {
        let citation = self.get_citation_by_identifier(identifier)
            .await?
            .ok_or_else(|| ValueError::EntityMissing(identifier.to_string()))?;
        Ok(self.add_citation_link(citation.id, resource_path).await?)
    }

    async fn get_citation_resource_set(
        &self,
        identifier: &str,
    ) -> Result<Option<CitationResourceSet>, BackendError> {
        if let Some(citation) = self.get_citation_by_identifier(identifier).await? {
            let resource_paths = self.list_citation_resources(identifier).await?;
            Ok(Some(CitationResourceSet {
                resource_paths,
                citation,
            }))
        } else {
            Ok(None)
        }
    }
}
