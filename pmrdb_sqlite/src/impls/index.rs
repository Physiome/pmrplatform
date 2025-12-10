use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    index::{
        IdxKind,
        IndexResourceSet,
        IndexTerms,
        traits::IndexBackend,
    }
};

use crate::SqliteBackend;

// Postgresql version should look like this instead:
//
// WITH q AS (
//     INSERT INTO idx_kind (
//         description
//     )
//     VALUES ( ?1 )
//     ON CONFLICT (description) DO NOTHING
//     RETURNING id
// )
// SELECT * from q
// UNION
//     SELECT id FROM idx_kind WHERE description = ?1
//
// This will be the case for similar functions.

async fn resolve_kind_sqlite(
    backend: &SqliteBackend,
    kind: &str,
) -> Result<i64, BackendError> {
    let id = sqlx::query!(
        r#"
INSERT INTO idx_kind (
    description
)
VALUES ( ?1 )
ON CONFLICT(description) DO UPDATE SET
    description = description
RETURNING id
        "#,
        kind,
    )
    .map(|row| row.id)
    .fetch_one(&*backend.pool)
    .await?;

    Ok(id)
}

async fn resolve_idx_entry_sqlite(
    backend: &SqliteBackend,
    idx_kind_id: i64,
    term: &str,
) -> Result<i64, BackendError> {
    let id = sqlx::query!(
        r#"
INSERT INTO idx_entry (
    idx_kind_id,
    term
)
VALUES ( ?1, ?2 )
ON CONFLICT(idx_kind_id, term) DO UPDATE SET
    idx_kind_id = idx_kind_id,
    term = term
RETURNING id
        "#,
        idx_kind_id,
        term,
    )
    .map(|row| row.id)
    .fetch_one(&*backend.pool)
    .await?;

    Ok(id)
}

async fn add_idx_entry_link_sqlite(
    backend: &SqliteBackend,
    idx_entry_id: i64,
    resource_path: &str,
) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
INSERT INTO idx_entry_link (
    idx_entry_id,
    resource_path
)
VALUES ( ?1, ?2 )
ON CONFLICT(idx_entry_id, resource_path) DO NOTHING
        "#,
        idx_entry_id,
        resource_path,
    )
    .execute(&*backend.pool)
    .await?;

    Ok(())
}

async fn forget_resource_path_sqlite(
    backend: &SqliteBackend,
    kind: Option<&str>,
    resource_path: &str,
) -> Result<(), BackendError> {
    match kind {
        None => {
            sqlx::query!(
                r#"
DELETE FROM idx_entry_link
WHERE
    resource_path = ?1
                "#,
                resource_path,
            )
                .execute(&*backend.pool)
                .await?;
        }
        Some(kind) => {
            let idx_kind = match get_idx_kind_sqlite(backend, kind).await? {
                Some(idx_kind) => idx_kind,
                None => return Ok(()),
            };
            sqlx::query!(
                r#"
DELETE FROM idx_entry_link
WHERE
    resource_path = ?1 AND idx_entry_id IN (
        SELECT
            id
        FROM
            idx_entry
        WHERE
            idx_kind_id = ?2
    )
                "#,
                resource_path,
                idx_kind.id,
            )
                .execute(&*backend.pool)
                .await?;
        }
    }

    Ok(())
}

async fn list_kinds_sqlite(
    backend: &SqliteBackend,
) -> Result<Vec<String>, BackendError> {
    let recs = sqlx::query!(
        r#"
SELECT
    description
FROM
    idx_kind
        "#,
    )
    .map(|row| row.description)
    .fetch_all(&*backend.pool)
    .await?;

    Ok(recs)
}

async fn get_idx_kind_sqlite(
    backend: &SqliteBackend,
    kind: &str,
) -> Result<Option<IdxKind>, BackendError> {
     Ok(sqlx::query_as!(
        IdxKind,
        r#"
SELECT
    id,
    description
FROM
    idx_kind
WHERE
    description = ?1
        "#,
        kind,
    )
    .fetch_optional(&*backend.pool)
    .await?)
}

async fn list_terms_sqlite(
    backend: &SqliteBackend,
    kind: &str,
) -> Result<Option<IndexTerms>, BackendError> {
    let idx_kind = match get_idx_kind_sqlite(backend, kind).await? {
        Some(idx_kind) => idx_kind,
        None => return Ok(None),
    };

    let terms = sqlx::query!(
        r#"
SELECT
    term
FROM
    idx_entry
WHERE
    idx_kind_id = ?1
        "#,
        idx_kind.id,
    )
    .map(|row| row.term)
    .fetch_all(&*backend.pool)
    .await?;

    Ok(Some(IndexTerms {
        kind: idx_kind,
        terms,
    }))
}

async fn list_resources_sqlite(
    backend: &SqliteBackend,
    kind: &str,
    term: &str,
) -> Result<Option<IndexResourceSet>, BackendError> {
    let idx_kind = match get_idx_kind_sqlite(backend, kind).await? {
        Some(idx_kind) => idx_kind,
        None => return Ok(None),
    };

    let idx_entry_id = match sqlx::query!(
        r#"
SELECT
    id
FROM
    idx_entry
WHERE
    idx_kind_id = ?1
AND
    term = ?2
        "#,
        idx_kind.id,
        term,
    )
    .map(|row| row.id)
    .fetch_optional(&*backend.pool)
    .await? {
        Some(idx_entry_id) => idx_entry_id,
        None => return Ok(None),
    };

    let resource_paths = sqlx::query!(
        r#"
SELECT
    resource_path
FROM
    idx_entry_link
WHERE
    idx_entry_id = ?1
        "#,
        idx_entry_id,
    )
    .map(|row| row.resource_path)
    .fetch_all(&*backend.pool)
    .await?;

    Ok(Some(IndexResourceSet {
        kind: idx_kind,
        term: term.to_string(),
        resource_paths,
    }))
}

#[async_trait]
impl IndexBackend for SqliteBackend {
    async fn resolve_kind(
        &self,
        kind: &str,
    ) -> Result<i64, BackendError> {
        resolve_kind_sqlite(self, kind).await
    }

    async fn resolve_idx_entry(
        &self,
        idx_kind_id: i64,
        term: &str,
    ) -> Result<i64, BackendError> {
        resolve_idx_entry_sqlite(self, idx_kind_id, term).await
    }

    async fn add_idx_entry_link(
        &self,
        idx_entry_id: i64,
        resource_path: &str,
    ) -> Result<(), BackendError> {
        add_idx_entry_link_sqlite(self, idx_entry_id, resource_path).await
    }

    async fn forget_resource_path(
        &self,
        kind: Option<&str>,
        resource_path: &str,
    ) -> Result<(), BackendError> {
        forget_resource_path_sqlite(self, kind, resource_path).await
    }

    async fn list_kinds(&self) -> Result<Vec<String>, BackendError> {
        list_kinds_sqlite(self).await
    }

    async fn list_terms(
        &self,
        kind: &str,
    ) -> Result<Option<IndexTerms>, BackendError> {
        list_terms_sqlite(self, kind).await
    }

    async fn list_resources(
        &self,
        kind: &str,
        term: &str,
    ) -> Result<Option<IndexResourceSet>, BackendError> {
        list_resources_sqlite(self, kind, term).await
    }

}

#[cfg(test)]
pub(crate) mod testing {
    use pmrcore::{
        platform::PlatformConnector as _,
        index::traits::IndexBackend,
    };
    use crate::SqliteBackend;

    #[async_std::test]
    async fn test_empty_index() -> anyhow::Result<()> {
        let backend = SqliteBackend::pc("sqlite::memory:".into())
            .await
            .map_err(anyhow::Error::from_boxed)?;
        assert!(backend.list_resources("no_such_kind", "no_such_term").await?.is_none());
        assert!(backend.list_terms("no_such_kind").await?.is_none());
        assert!(backend.list_kinds().await?.is_empty());

        Ok(())
    }

    #[async_std::test]
    async fn test_foundation() -> anyhow::Result<()> {
        let backend = SqliteBackend::pc("sqlite::memory:".into())
            .await
            .map_err(anyhow::Error::from_boxed)?;

        let id1 = backend.resolve_kind("test_kind").await?;
        let id2 = backend.resolve_kind("test_kind").await?;
        assert_eq!(id1, id2);

        let id3 = backend.resolve_idx_entry(id1, "test_term").await?;
        let id4 = backend.resolve_idx_entry(id1, "test_term").await?;
        assert_eq!(id3, id4);

        backend.add_idx_entry_link(id3, "resource1").await?;
        backend.add_idx_entry_link(id3, "resource1").await?;

        assert_eq!(
            backend.list_kinds().await?,
            vec!["test_kind".to_string()],
        );
        assert_eq!(
            backend.list_terms("test_kind").await?.unwrap().terms,
            vec!["test_term".to_string()],
        );
        assert_eq!(
            backend.list_resources("test_kind", "test_term").await?.unwrap().resource_paths,
            vec!["resource1".to_string()],
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_index_resource() -> anyhow::Result<()> {
        let backend = SqliteBackend::pc("sqlite::memory:".into())
            .await
            .map_err(anyhow::Error::from_boxed)?;

        backend.index_resource("keyword", "/test/resource", &mut [].into_iter()).await?;
        assert_eq!(
            backend.list_kinds().await?,
            vec!["keyword".to_string()],
        );
        assert!(backend.list_terms("keyword").await?.unwrap().terms.is_empty());

        backend.index_resource("title", "/test/resource", &mut [
            "Test Resource",
        ].into_iter()).await?;
        backend.index_resource("keyword", "/test/resource", &mut [
            "hello",
            "world",
        ].into_iter()).await?;
        assert_eq!(
            backend.list_kinds().await?,
            vec!["keyword".to_string(), "title".to_string()],
        );
        assert_eq!(
            backend.list_terms("keyword").await?.unwrap().terms,
            vec!["hello".to_string(), "world".to_string()],
        );
        // this isn't exactly highly ergonomic, but it does distinguish between a term
        // that has been seen vs. unknown.
        assert_eq!(
            backend.list_resources("keyword", "hello").await?.unwrap().resource_paths,
            vec!["/test/resource".to_string()],
        );
        assert!(backend.list_resources("keyword", "Test Resource").await?.is_none());

        backend.forget_resource_path(Some("keyword"), "/test/resource").await?;
        assert!(backend.list_resources("keyword", "hello").await?.unwrap().resource_paths.is_empty());
        assert_eq!(
            backend.list_resources("title", "Test Resource").await?.unwrap().resource_paths,
            vec!["/test/resource".to_string()],
        );
        backend.forget_resource_path(None, "/test/resource").await?;
        assert!(backend.list_resources("title", "Test Resource").await?.unwrap().resource_paths.is_empty());
        // TODO should clean up terms that have no records?

        Ok(())
    }

}
