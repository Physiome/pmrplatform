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

fn query_items<F>(
    store: &Store,
    query: &'static str,
    var: &'static str,
    value: Option<&str>,
    formatter: F,
    literal: bool,
    iri: bool,
) -> Result<Vec<String>, RdfIndexerError>
where
    F: Fn(&str) -> String,
{
    let mut result = Vec::new();
    let mut query = SparqlEvaluator::new().parse_query(query)?;

    if let Some(value) = value {
        let iri = Iri::parse(BASE_IRI)?.resolve(value)?;
        query = query.substitute_variable(
            Variable::new("node").expect("specified static value must parse correctly"),
            NamedNodeRef::new_unchecked(&iri),
        );
    }

    if let QueryResults::Solutions(solutions) = query.on_store(&store).execute()? {
        for solution in solutions {
            if let Ok(solution) = solution {
                if literal && let Some(Term::Literal(literal)) = solution.get(var) {
                    result.push(formatter(literal.value()));
                }
                if iri && let Some(Term::NamedNode(literal)) = solution.get(var) {
                    result.push(formatter(literal.as_str()));
                }
            }
        }
    }
    Ok(result)
}

fn query_literals<F>(
    store: &Store,
    query: &'static str,
    var: &'static str,
    value: Option<&str>,
    formatter: F,
) -> Result<Vec<String>, RdfIndexerError>
where
    F: Fn(&str) -> String,
{
    query_items(store, query, var, value, formatter, true, false)
}

fn query_iris<F>(
    store: &Store,
    query: &'static str,
    var: &'static str,
    value: Option<&str>,
    formatter: F,
) -> Result<Vec<String>, RdfIndexerError>
where
    F: Fn(&str) -> String,
{
    query_items(store, query, var, value, formatter, false, true)
}

pub fn keywords(store: &Store) -> Result<Vec<String>, RdfIndexerError> {
    query_literals(
        store,
        r#"
            PREFIX bqs: <http://www.cellml.org/bqs/1.0#>
            PREFIX dc: <http://purl.org/dc/elements/1.1/>
            PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

            SELECT ?cmetaid ?value
            WHERE {
                ?cmetaid bqs:reference ?bqs .
                ?bqs dc:subject [ rdf:value ?container ] .
                ?container ?li ?value .
            }
        "#,
        "value",
        None,
        str::to_string,
    )
}

pub fn pubmed_id(store: &Store) -> Result<Vec<String>, RdfIndexerError> {
    query_literals(
        store,
        r#"
            PREFIX bqs: <http://www.cellml.org/bqs/1.0#>

            SELECT ?ref ?pmid
            WHERE {
                ?node bqs:reference ?ref .
                ?ref bqs:JournalArticle ?article .
                OPTIONAL { ?ref bqs:Pubmed_id ?pmid } .
            }
        "#,
        "pmid",
        None,
        |pmid| format!("pmid:{pmid}"),
    )
}

/// Return the dc:title, optionally constrained the results from the specified node.
pub fn dc_title(store: &Store, node: Option<&str>) -> Result<Vec<String>, RdfIndexerError> {
    query_literals(
        store,
        r#"
            PREFIX dc: <http://purl.org/dc/elements/1.1/>

            SELECT ?node ?title
            WHERE {
                ?node dc:title ?title .
            }
        "#,
        "title",
        node,
        str::to_string,
    )
}

/// Return the license specified by the graph
pub fn license(store: &Store) -> Result<Option<String>, RdfIndexerError> {
    Ok(query_iris(
        store,
        r#"
            PREFIX dcterms: <http://purl.org/dc/terms/>

            SELECT ?node ?license
            WHERE {
                ?node dcterms:license ?license .
            }
        "#,
        "license",
        Some(""),
        str::to_string,
    )?
    .get(0)
    .map(Clone::clone))
}
