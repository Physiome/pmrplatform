use async_trait::async_trait;
use crate::error::BackendError;

use super::HexId;

/// This particular backend is here to mimic the global incremental identifier
/// that the original PMR2 had.
#[async_trait]
pub trait GenAliasBackend {
    async fn next(&self) -> Result<HexId, BackendError>;
}
