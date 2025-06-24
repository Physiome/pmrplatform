use async_trait::async_trait;
use crate::{
    error::BackendError,
    alias::Alias
};

#[async_trait]
pub trait AliasBackend {
    async fn add_alias(
        &self,
        kind: &str,
        kind_id: i64,
        alias: &str
    ) -> Result<i64, BackendError>;
    async fn get_aliases(
        &self,
        kind: &str,
        kind_id: i64,
    ) -> Result<Vec<Alias>, BackendError>;
    async fn resolve_alias(
        &self,
        kind: &str,
        alias: &str,
    ) -> Result<Option<i64>, BackendError>;
}
