use async_trait::async_trait;
use pmrcore::{
    error::BackendError,
    citation::{
        Citation,
        CitationAuthor,
        traits::CitationBackend,
    }
};

use crate::SqliteBackend;

async fn add_author_sqlite(
    backend: &SqliteBackend,
    citation_id: &str,
    ordering: i64,
    author: &CitationAuthor,
) -> Result<(), BackendError> {
    sqlx::query!(
        r#"
INSERT INTO citation_author (
    citation_id,
    family,
    given,
    other,
    ordering
)
VALUES ( ?1, ?2, ?3, ?4, ?5 )
        "#,
        citation_id,
        author.family,
        author.given,
        author.other,
        ordering,
    )
    .execute(&*backend.pool)
    .await?;

    Ok(())
}

async fn add_citation_sqlite(
    backend: &SqliteBackend,
    citation: &Citation,
) -> Result<(), BackendError> {
    match sqlx::query!(
        r#"
INSERT INTO citation (
    id,
    title,
    journal,
    volume,
    first_page,
    last_page,
    issued
)
VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7 )
        "#,
        citation.id,
        citation.title,
        citation.journal,
        citation.volume,
        citation.first_page,
        citation.last_page,
        citation.issued,
    )
    .execute(&*backend.pool)
    .await {
        Ok(_) => (),
        Err(e) => {
            return match e.as_database_error() {
                // TODO should verify whether or not this particular citation is identical to the
                // one already stored in the database.
                Some(db_e) if db_e.is_unique_violation() => Ok(()),
                _ => Err(e)?,
            };
        }
    };


    for (i, author) in citation.authors.iter().enumerate() {
        add_author_sqlite(backend, &citation.id, i as i64, author).await?;
    }

    Ok(())
}

async fn get_authors_sqlite(
    backend: &SqliteBackend,
    citation_id: &str,
) -> Result<Vec<CitationAuthor>, BackendError> {
    let authors = sqlx::query_as!(
        CitationAuthor,
        r#"
SELECT
    family,
    given,
    other
FROM
    citation_author
WHERE
    citation_id = ?1
ORDER BY
    ordering
        "#,
        citation_id,
    )
    .fetch_all(&*backend.pool)
    .await?;

    Ok(authors)
}

async fn get_citation_by_identifier_sqlite(
    backend: &SqliteBackend,
    id: &str,
) -> Result<Option<Citation>, BackendError> {
    let citation = sqlx::query!(
        r#"
SELECT
    id,
    title,
    journal,
    volume,
    first_page,
    last_page,
    issued
FROM
    citation
WHERE
    id = ?1
        "#,
        id,
    )
    .map(|citation| Box::pin(async {
        Ok::<_, BackendError>(Citation {
            authors: get_authors_sqlite(backend, &citation.id).await?,
            id: citation.id,
            title: citation.title,
            journal: citation.journal,
            volume: citation.volume,
            first_page: citation.first_page,
            last_page: citation.last_page,
            issued: citation.issued,
        })
    }))
    .fetch_optional(&*backend.pool)
    .await?;

    match citation {
        Some(citation) => Ok(Some(citation.await?)),
        None => Ok(None)
    }
}

async fn list_citations_sqlite(
    backend: &SqliteBackend,
) -> Result<Vec<Citation>, BackendError> {
    let mut citations = Vec::new();
    let futures = sqlx::query!(
        r#"
SELECT
    id,
    title,
    journal,
    volume,
    first_page,
    last_page,
    issued
FROM
    citation
ORDER BY
    title
        "#,
    )
    .map(|citation| Box::pin(async {
        Ok::<_, BackendError>(Citation {
            authors: get_authors_sqlite(backend, &citation.id).await?,
            id: citation.id,
            title: citation.title,
            journal: citation.journal,
            volume: citation.volume,
            first_page: citation.first_page,
            last_page: citation.last_page,
            issued: citation.issued,
        })
    }))
    .fetch_all(&*backend.pool)
    .await?;

    for fut in futures {
        citations.push(fut.await?);
    }

    Ok(citations)
}

#[async_trait]
impl CitationBackend for SqliteBackend {
    async fn add_citation(
        &self,
        citation: &Citation,
    ) -> Result<(), BackendError> {
        add_citation_sqlite(
            &self,
            citation,
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

        let citation = Citation {
            id: "urn:foo:example".to_string(),
            title: "Example Title".to_string(),
            authors: vec![ CitationAuthor {
                family: "Family4".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Family3".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Family1".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Family2".to_string(),
                .. Default::default()
            }],
            .. Default::default()
        };
        backend.add_citation(&citation).await?;
        let all_citations = backend.list_citations().await?;
        assert_eq!(all_citations.len(), 1);
        assert_eq!(all_citations.get(0), Some(citation).as_ref());
        Ok(())
    }

    #[async_std::test]
    async fn test_duplicate() -> anyhow::Result<()> {
        let backend = SqliteBackend::pc("sqlite::memory:".into())
            .await
            .map_err(anyhow::Error::from_boxed)?;

        let citation = Citation {
            id: "urn:foo:example".to_string(),
            title: "Example Title".to_string(),
            authors: vec![ CitationAuthor {
                family: "Family4".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Family3".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Family1".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Family2".to_string(),
                .. Default::default()
            }],
            .. Default::default()
        };
        // same dupe
        backend.add_citation(&citation).await?;
        // diff dupe
        let dupe_citation = Citation {
            // duplicate id does nothing
            id: "urn:foo:example".to_string(),
            title: "Ignored Title".to_string(),
            authors: vec![ CitationAuthor {
                family: "Ignored4".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Ignored3".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Ignored1".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Ignored2".to_string(),
                .. Default::default()
            }],
            .. Default::default()
        };
        backend.add_citation(&dupe_citation).await?;

        let all_citations = backend.list_citations().await?;
        assert_eq!(all_citations.len(), 1);
        assert_eq!(all_citations.get(0), Some(citation).as_ref());
        Ok(())
    }

    #[async_std::test]
    async fn test_multi() -> anyhow::Result<()> {
        let backend = SqliteBackend::pc("sqlite::memory:".into())
            .await
            .map_err(anyhow::Error::from_boxed)?;

        let citation1 = Citation {
            id: "urn:foo:example_1".to_string(),
            title: "First Example".to_string(),
            authors: vec![ CitationAuthor {
                family: "Family4".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Family3".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Family1".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Family2".to_string(),
                .. Default::default()
            }],
            .. Default::default()
        };
        let citation2 = Citation {
            id: "urn:foo:example_2".to_string(),
            title: "Example Two".to_string(),
            authors: vec![ CitationAuthor {
                family: "Family1".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Family3".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Family2".to_string(),
                .. Default::default()
            }, CitationAuthor {
                family: "Family5".to_string(),
                .. Default::default()
            }],
            .. Default::default()
        };
        backend.add_citation(&citation1).await?;
        backend.add_citation(&citation2).await?;
        let all_citations = backend.list_citations().await?;
        assert_eq!(all_citations.len(), 2);
        // check ordered by title
        assert_eq!(all_citations.get(0), Some(citation2).as_ref());
        assert_eq!(all_citations.get(1), Some(citation1).as_ref());
        Ok(())
    }

}
