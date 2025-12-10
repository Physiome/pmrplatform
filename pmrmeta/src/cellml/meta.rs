use oxigraph::{
    model::Term,
    sparql::{QueryResults, SparqlEvaluator},
    store::Store,
};

use crate::error::RdfIndexerError;

pub fn query_keywords(store: &Store) -> Result<Vec<String>, RdfIndexerError> {
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

pub fn query_pubmed_id(store: &Store) -> Result<Vec<String>, RdfIndexerError> {
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
