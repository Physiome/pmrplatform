use async_trait::async_trait;
use crate::{
    error::BackendError,
    alias,
};

#[async_trait]
pub trait AliasBackend {
    async fn add_alias(
        &self,
        kind: &str,
        kind_id: i64,
        alias: &str
    ) -> Result<(), BackendError>;
    async fn get_alias(
        &self,
        kind: &str,
        kind_id: i64,
    ) -> Result<Option<String>, BackendError>;
    async fn get_aliases(
        &self,
        kind: &str,
        kind_id: i64,
    ) -> Result<Vec<alias::Alias>, BackendError>;
    async fn resolve_alias(
        &self,
        kind: &str,
        alias: &str,
    ) -> Result<Option<i64>, BackendError>;
    async fn aliases_by_kind(
        &self,
        kind: &str,
    ) -> Result<Vec<(String, i64)>, BackendError>;
}

#[async_trait]
pub trait Alias<'a, T> {
    fn kind(&self) -> &str;
    fn kind_id(&self) -> i64;
    fn alias(&self) -> &str;
    fn created_ts(&self) -> i64;
    fn aliased(&'a self) -> Option<&'a T>;
}
