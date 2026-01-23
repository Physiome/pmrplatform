use async_trait::async_trait;
use crate::{
    citation::Citation,
    error::BackendError,
};

#[async_trait]
pub trait CitationBackend {
    async fn add_citation(
        &self,
        citation: &Citation,
    ) -> Result<(), BackendError>;

    /// Get a particular citation
    async fn get_citation_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<Citation>, BackendError>;

    /// returns the full listing of `Citation`
    async fn list_citations(
        &self,
    ) -> Result<Vec<Citation>, BackendError>;
}
