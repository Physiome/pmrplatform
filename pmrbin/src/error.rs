use thiserror::Error;
use oxigraph::{
    io::RdfParseError,
    model::IriParseError,
    sparql::{QueryEvaluationError, SparqlSyntaxError},
    store::StorageError,
};

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum RdfIndexerError {
    // TODO figure out how much info to actually include here
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    IriParseError(#[from] IriParseError),
    #[error(transparent)]
    QueryEvaluationError(#[from] QueryEvaluationError),
    #[error(transparent)]
    RdfParseError(#[from] RdfParseError),
    #[error(transparent)]
    RdfStorageError(#[from] StorageError),
    #[error(transparent)]
    SparqlSyntaxError(#[from] SparqlSyntaxError),
    #[error(transparent)]
    XeeDocumentsError(#[from] xee_xpath::error::DocumentsError),
    #[error(transparent)]
    XeeErrorValue(#[from] xee_xpath::error::ErrorValue),
    #[error(transparent)]
    XeeXpathError(#[from] xee_xpath::error::Error),
}
