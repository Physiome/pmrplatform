use oxigraph::{
    model::Term,
    sparql::{QueryResults, SparqlEvaluator},
    store::Store,
};

use crate::error::RdfIndexerError;

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
