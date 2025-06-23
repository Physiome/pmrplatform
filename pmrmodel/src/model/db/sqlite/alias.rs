use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    alias::{
        Alias,
        traits::AliasBackend,
    },
};

use crate::{
    backend::db::SqliteBackend,
    chrono::Utc,
};

#[async_trait]
impl AliasBackend for SqliteBackend {
    async fn add_alias(
        &self,
        kind: &str,
        kind_id: i64,
        alias: &str
    ) -> Result<i64, BackendError> {
        let ts = Utc::now().timestamp();
        let id = sqlx::query!(
            r#"
INSERT INTO alias ( kind, kind_id, alias, created_ts )
VALUES ( ?1, ?2, ?3, ?4 )
            "#,
            kind,
            kind_id,
            alias,
            ts,
        )
        .execute(&*self.pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    async fn get_aliases(
        &self,
        kind: &str,
        kind_id: i64,
    ) -> Result<Vec<Alias>, BackendError> {
        let recs = sqlx::query_as!(Alias,
            r#"
SELECT kind, kind_id, alias, created_ts
FROM alias
WHERE kind = ?1 AND kind_id = ?2
            "#,
            kind,
            kind_id,
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(recs)
    }

    async fn resolve_alias(
        &self,
        kind: &str,
        alias: &str,
    ) -> Result<i64, BackendError> {
        let rec = sqlx::query!(
            r#"
SELECT kind_id
FROM alias
WHERE kind = ?1 AND alias = ?2
            "#,
            kind,
            alias,
        )
        .map(|rec| rec.kind_id)
        .fetch_one(&*self.pool)
        .await?;
        Ok(rec)
    }
}
