use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    citation::{
        Citation,
        traits::CitationBackend,
    }
};

use crate::SqliteBackend;

async fn add_citation_sqlite(
    backend: &SqliteBackend,
    identifier: &str,
) -> Result<i64, BackendError> {
    let id = sqlx::query!(
        r#"
INSERT INTO citation (
    identifier
)
VALUES ( ?1 )
        "#,
        identifier,
    )
    .execute(&*backend.pool)
    .await?
    .last_insert_rowid();

    Ok(id)
}

async fn add_citation_link_sqlite(
    backend: &SqliteBackend,
    citation_id: i64,
    resource_path: &str,
) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
INSERT INTO citation_link (
    citation_id,
    resource_path
)
VALUES ( ?1, ?2 )
        "#,
        citation_id,
        resource_path,
    )
    .execute(&*backend.pool)
    .await?;

    Ok(())
}

async fn get_citation_by_identifier_sqlite(
    backend: &SqliteBackend,
    identifier: &str,
) -> Result<Option<Citation>, BackendError> {
    let recs = sqlx::query!(
        r#"
SELECT
    id,
    identifier
FROM
    citation
WHERE
    identifier = ?1
        "#,
        identifier,
    )
    .map(|row| Citation {
        id: row.id,
        identifier: row.identifier,
    })
    .fetch_optional(&*backend.pool)
    .await?;

    Ok(recs)
}

async fn list_citations_sqlite(
    backend: &SqliteBackend,
) -> Result<Vec<Citation>, BackendError> {
    let recs = sqlx::query!(
        r#"
SELECT
    id,
    identifier
FROM
    citation
ORDER BY
    id
        "#,
    )
    .map(|row| Citation {
        id: row.id,
        identifier: row.identifier,
    })
    .fetch_all(&*backend.pool)
    .await?;

    Ok(recs)
}

async fn list_citation_resources_sqlite(
    backend: &SqliteBackend,
    identifier: &str,
) -> Result<Vec<String>, BackendError> {
    let result = sqlx::query!(
        r#"
SELECT
    resource_path
FROM
    citation_link
WHERE
    citation_id = (
        SELECT id
        FROM citation
        WHERE identifier = ?1
    )
ORDER BY
    resource_path
        "#,
        identifier,
    )
    .map(|row| row.resource_path)
    .fetch_all(&*backend.pool)
    .await?;

    Ok(result)
}

#[async_trait]
impl CitationBackend for SqliteBackend {
    async fn add_citation(
        &self,
        identifier: &str,
    ) -> Result<i64, BackendError> {
        add_citation_sqlite(
            &self,
            identifier,
        ).await
    }

    async fn get_citation_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<Option<Citation>, BackendError> {
        get_citation_by_identifier_sqlite(&self, identifier).await
    }

    async fn list_citations(
        &self,
    ) -> Result<Vec<Citation>, BackendError> {
        list_citations_sqlite(&self).await
    }

    async fn add_citation_link(
        &self,
        citation_id: i64,
        resource_path: &str,
    ) -> Result<(), BackendError> {
        add_citation_link_sqlite(
            &self,
            citation_id,
            resource_path,
        ).await
    }

    /// returns the resource string identifiers for the given citation identifier
    async fn list_citation_resources(
        &self,
        identifier: &str,
    ) -> Result<Vec<String>, BackendError> {
        list_citation_resources_sqlite(
            &self,
            identifier,
        ).await
    }

}

#[cfg(test)]
pub(crate) mod testing {
    use pmrcore::{
        platform::PlatformConnector as _,
        citation::traits::CitationBackend,
    };
    use crate::SqliteBackend;
    use super::*;

    #[async_std::test]
    async fn test_basic() -> anyhow::Result<()> {
        let backend = SqliteBackend::pc("sqlite::memory:".into())
            .await
            .map_err(anyhow::Error::from_boxed)?;

        backend.add_citation("urn:example:citation:unused").await?;
        let id1 = backend.add_citation("urn:example:citation1").await?;
        let id2 = backend.add_citation("urn:example:citation2").await?;

        backend.add_citation_link(id1, "http://example.com/model/1a").await?;
        backend.add_citation_link(id1, "http://example.com/model/1b").await?;

        backend.add_citation_link(id2, "http://example.com/model/2c").await?;
        backend.add_citation_link(id2, "http://example.com/model/2d").await?;
        backend.add_citation_link(id2, "http://example.com/model/2e").await?;

        assert_eq!(backend.list_citations().await?.len(), 3);

        assert!(backend.get_citation_by_identifier("no_such_resource").await?.is_none());
        assert_eq!(backend.list_citation_resources("no_such_resource").await?.len(), 0);

        assert_eq!(
            backend.get_citation_by_identifier("urn:example:citation:unused").await?.unwrap(),
            Citation {
                id: 1,
                identifier: "urn:example:citation:unused".to_string(),
            }
        );
        assert_eq!(backend.list_citation_resources("urn:example:citation:unused").await?.len(), 0);

        assert_eq!(backend.list_citation_resources("urn:example:citation1").await?.len(), 2);
        assert_eq!(backend.list_citation_resources("urn:example:citation2").await?.len(), 3);

        let result = backend.get_citation_resource_set("urn:example:citation2").await?
            .expect("has a result");
        assert_eq!(result.citation.id, 3);
        assert_eq!(result.resource_paths.len(), 3);

        Ok(())
    }

}
