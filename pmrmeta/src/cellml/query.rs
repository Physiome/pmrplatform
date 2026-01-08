use oxiri::Iri;
use oxigraph::{
    model::{NamedNodeRef, Term},
    sparql::{QueryResults, SparqlEvaluator, Variable},
    store::Store,
};

use crate::{
    error::RdfIndexerError,
    read::BASE_IRI,
};

pub fn keywords(store: &Store) -> Result<Vec<String>, RdfIndexerError> {
    let mut result = Vec::new();
    if let QueryResults::Solutions(solutions) = SparqlEvaluator::new()
        .parse_query(r#"
            PREFIX bqs: <http://www.cellml.org/bqs/1.0#>
            PREFIX dc: <http://purl.org/dc/elements/1.1/>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

            SELECT ?cmetaid ?value
            WHERE {
                ?cmetaid bqs:reference ?bqs .
                ?bqs dc:subject [ rdf:value ?container ] .
                ?container ?li ?value .
            }
        "#)?
        .on_store(&store)
        .execute()?
    {
        for solution in solutions {
            if let Ok(solution) = solution {
                if let Some(Term::Literal(literal)) = solution.get("value") {
                    result.push(literal.value().to_string());
                }
            }
        }
    }
    Ok(result)
}

pub fn pubmed_id(store: &Store) -> Result<Vec<String>, RdfIndexerError> {
    let mut result = Vec::new();
    if let QueryResults::Solutions(solutions) = SparqlEvaluator::new()
        .parse_query(r#"
            PREFIX bqs: <http://www.cellml.org/bqs/1.0#>

            SELECT ?ref ?pmid
            WHERE {
                ?node bqs:reference ?ref .
                ?ref bqs:JournalArticle ?article .
                OPTIONAL { ?ref bqs:Pubmed_id ?pmid } .
            }
        "#)?
        .on_store(&store)
        .execute()?
    {
        for solution in solutions {
            if let Ok(solution) = solution {
                if let Some(Term::Literal(literal)) = solution.get("pmid") {
                    result.push(format!("pmid:{}", literal.value()));
                }
            }
        }
    }
    Ok(result)
}

/// Return the dc:title, optionally constrained the results from the specified node.
pub fn dc_title(store: &Store, node: Option<&str>) -> Result<Vec<String>, RdfIndexerError> {
    let mut result = Vec::new();
    let mut query = SparqlEvaluator::new()
        .parse_query(r#"
            PREFIX dc: <http://purl.org/dc/elements/1.1/>

            SELECT ?node ?title
            WHERE {
                ?node dc:title ?title .
            }
        "#)?;

    if let Some(node) = node {
        let iri = Iri::parse(BASE_IRI)?.resolve(node)?;
        query = query.substitute_variable(
            Variable::new("node").expect("static correct value shouldn't fail"),
            NamedNodeRef::new_unchecked(&iri),
        );
    }

    if let QueryResults::Solutions(solutions) = query.on_store(&store).execute()? {
        for solution in solutions {
            if let Ok(solution) = solution {
                if let Some(Term::Literal(literal)) = solution.get("title") {
                    result.push(literal.value().to_string());
                }
            }
        }
    }
    Ok(result)
}
