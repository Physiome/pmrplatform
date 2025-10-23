use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    alias::{
        Alias,
        traits::AliasBackend,
    },
};
use sqlx::{QueryBuilder, Row, Sqlite};

use crate::{
    SqliteBackend,
    chrono::Utc,
};

#[async_trait]
impl AliasBackend for SqliteBackend {
    async fn add_alias(
        &self,
        kind: &str,
        kind_id: i64,
        alias: &str
    ) -> Result<(), BackendError> {
        let ts = Utc::now().timestamp();
        sqlx::query!(
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
        .await?;

        Ok(())
    }

    async fn get_alias(
        &self,
        kind: &str,
        kind_id: i64,
    ) -> Result<Option<String>, BackendError> {
        let rec = sqlx::query!(
            r#"
SELECT alias
FROM alias
WHERE kind = ?1 AND kind_id = ?2
ORDER BY created_ts
LIMIT 1
            "#,
            kind,
            kind_id,
        )
        .map(|rec| rec.alias)
        .fetch_optional(&*self.pool)
        .await?;
        Ok(rec)
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
    ) -> Result<Option<i64>, BackendError> {
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
        .fetch_optional(&*self.pool)
        .await?;
        Ok(rec)
    }

    async fn aliases_by_kind(
        &self,
        kind: &str,
    ) -> Result<Vec<(String, i64)>, BackendError> {
        let recs = sqlx::query!(
            r#"
SELECT alias, kind_id
FROM alias
WHERE kind = ?1
            "#,
            kind,
        )
        .map(|rec| (rec.alias, rec.kind_id))
        .fetch_all(&*self.pool)
        .await?;
        Ok(recs)
    }

    async fn aliases_by_kind_ids(
        &self,
        kind: &str,
        ids: &[i64],
    ) -> Result<Vec<(String, i64)>, BackendError> {
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(r#"
SELECT alias, kind_id
FROM alias
WHERE kind = "#);
        query_builder.push_bind(kind);
        query_builder.push("AND kind_id IN (");

        let mut separated = query_builder.separated(", ");
        for id in ids.iter() {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");

        let recs = query_builder
            .build()
            .try_map(|rec| Ok((rec.try_get("alias")?, rec.try_get("kind_id")?)))
            .fetch_all(&*self.pool)
            .await?;

        Ok(recs)
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use pmrcore::{
        platform::PlatformConnector as _,
        alias::{
            Alias,
            traits::AliasBackend,
        },
    };
    use test_pmr::chrono::set_timestamp;
    use crate::SqliteBackend;

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::mc("sqlite::memory:".into())
            .await
            .map_err(anyhow::Error::from_boxed)?;
        backend.add_alias("workspace", 1, "test_workspace").await?;
        backend.add_alias("exposure", 1, "main_exposure").await?;
        backend.add_alias("exposure", 2, "alternate_exposure").await?;

        let answer = Alias {
            kind: "workspace".to_string(),
            kind_id: 1,
            alias: "test_workspace".to_string(),
            created_ts: 1234567890,
        };

        let results = backend.get_aliases("workspace", 1).await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], answer);

        let result = backend.resolve_alias("workspace", "test_workspace").await?;
        assert_eq!(result, Some(answer.kind_id));

        let result = backend.resolve_alias("workspace", "does_not_exist").await?;
        assert_eq!(result, None);

        let mut results = backend.aliases_by_kind("exposure").await?;
        results.sort();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], ("alternate_exposure".to_string(), 2));
        assert_eq!(results[1], ("main_exposure".to_string(), 1));

        let results = backend.aliases_by_kind_ids("exposure", &[1]).await?;
        assert_eq!(results[0], ("main_exposure".to_string(), 1));

        let results = backend.aliases_by_kind_ids("exposure", &[1, 2]).await?;
        assert_eq!(results[0], ("main_exposure".to_string(), 1));
        assert_eq!(results[1], ("alternate_exposure".to_string(), 2));

        Ok(())
    }

    #[async_std::test]
    async fn test_get_alias() -> anyhow::Result<()> {
        let backend = SqliteBackend::mc("sqlite::memory:".into())
            .await
            .map_err(anyhow::Error::from_boxed)?;
        set_timestamp(987);
        backend.add_alias("workspace", 1, "test_alias").await?;
        set_timestamp(456);
        backend.add_alias("workspace", 1, "other_alias").await?;
        assert_eq!(
            backend.get_alias("workspace", 1).await?.as_deref(),
            Some("other_alias"),
        );
        assert!(backend.get_alias("exposure", 1).await?.is_none());
        Ok(())
    }
}
