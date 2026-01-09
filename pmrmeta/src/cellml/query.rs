use oxiri::Iri;
use oxigraph::{
    model::{NamedNodeRef, Term},
    sparql::{QueryResults, QuerySolution, SparqlEvaluator, Variable},
    store::Store,
};

use crate::{
    cellml::Citation,
    error::RdfIndexerError,
    read::BASE_IRI,
};

fn query_solutions<F, T>(
    store: &Store,
    query: &'static str,
    root_node: Option<(&'static str, &str)>,
    extractor: F,
) -> Result<Vec<T>, RdfIndexerError>
where
    F: Fn(QuerySolution) -> Option<T>,
{
    let mut result = Vec::new();
    let mut query = SparqlEvaluator::new().parse_query(query)?;

    if let Some((node_id, node_iri)) = root_node {
        let node_iri = Iri::parse(BASE_IRI)?.resolve(node_iri)?;
        query = query.substitute_variable(
            Variable::new(node_id).expect("specified static node_id must parse correctly"),
            NamedNodeRef::new_unchecked(&node_iri),
        );
    }

    if let QueryResults::Solutions(solutions) = query.on_store(&store).execute()? {
        for solution in solutions {
            if let Ok(solution) = solution {
                if let Some(value) = extractor(solution) {
                    result.push(value)
                }
            }
        }
    }
    Ok(result)
}

fn format_solution<F>(
    var_id: &'static str,
    formatter: F,
    literal: bool,
    iri: bool,
) -> impl Fn(QuerySolution) -> Option<String>
where
    F: Fn(&str) -> String,
{
    move |solution| {
        if literal && let Some(Term::Literal(literal)) = solution.get(var_id) {
            Some(formatter(literal.value()))
        }
        else if iri && let Some(Term::NamedNode(literal)) = solution.get(var_id) {
            Some(formatter(literal.as_str()))
        }
        else {
            None
        }
    }
}


fn query_items<F>(
    store: &Store,
    query: &'static str,
    root_node: Option<(&'static str, &str)>,
    var_id: &'static str,
    formatter: F,
    literal: bool,
    iri: bool,
) -> Result<Vec<String>, RdfIndexerError>
where
    F: Fn(&str) -> String,
{
    query_solutions(store, query, root_node, format_solution(var_id, formatter, literal, iri))
}

fn query_literals<F>(
    store: &Store,
    query: &'static str,
    root_node: Option<(&'static str, &str)>,
    var_id: &'static str,
    formatter: F,
) -> Result<Vec<String>, RdfIndexerError>
where
    F: Fn(&str) -> String,
{
    query_items(store, query, root_node, var_id, formatter, true, false)
}

fn query_iris<F>(
    store: &Store,
    query: &'static str,
    root_node: Option<(&'static str, &str)>,
    var_id: &'static str,
    formatter: F,
) -> Result<Vec<String>, RdfIndexerError>
where
    F: Fn(&str) -> String,
{
    query_items(store, query, root_node, var_id, formatter, false, true)
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
        None,
        "value",
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
        None,
        "pmid",
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
        node.map(|node| ("node", node)),
        "title",
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
        Some(("node", "")),
        "license",
        str::to_string,
    )?
    .get(0)
    .map(Clone::clone))
}

/// Return the citation from the specified node
///
/// Typically this node will be resolved from the cmeta:id defined in the CellML file.
pub fn citation(store: &Store, node: Option<&str>) -> Result<Vec<Citation>, RdfIndexerError> {
    query_solutions(
        store,
        r#"
        SELECT ?ref ?pmid ?title ?journal ?volume ?first_page ?last_page ?pdate
            ?croot
        WHERE {
            ?node bqs:reference ?ref .
            ?ref bqs:JournalArticle ?article .
            OPTIONAL { ?article dc:creator ?croot } .
            OPTIONAL { ?article bqs:Journal [ dc:title ?journal ] } .
            OPTIONAL { ?ref bqs:Pubmed_id ?pmid } .
            OPTIONAL { ?article dc:title ?title } .
            OPTIONAL { ?article bqs:volume ?volume } .
            OPTIONAL { ?article bqs:first_page ?first_page } .
            OPTIONAL { ?article bqs:last_page ?last_page } .
            OPTIONAL { ?article dcterms:issued [ dcterms:W3CDTF ?pdate ] } .
        }
        "#,
        node.map(|node| ("node", node)),
        |solution| {
            let mut citation = Citation::default();
            if let Some(Term::Literal(literal)) = solution.get("pmid") {
                citation.id = Some(format!("urn:miriam:pubmed:{}", literal.value()))
            }
            if let Some(Term::Literal(literal)) = solution.get("title") {
                citation.title = Some(literal.value().to_string())
            }
            if let Some(Term::Literal(literal)) = solution.get("journal") {
                citation.journal = Some(literal.value().to_string())
            }
            Some(citation)
        },
    )
}
