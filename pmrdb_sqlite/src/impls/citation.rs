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
        assert!(backend.add_citation("urn:example:citation:unused").await.is_err());
        backend.add_citation("urn:example:citation1").await?;
        backend.add_citation("urn:example:citation2").await?;
        assert_eq!(backend.list_citations().await?.len(), 3);
        assert_eq!(
            backend.get_citation_by_identifier("urn:example:citation:unused").await?.unwrap(),
            Citation {
                id: 1,
                identifier: "urn:example:citation:unused".to_string(),
            }
        );
        Ok(())
    }

}
