use oxiri::{IriParseError, IriRef};
use oxigraph::{
    model::{NamedNode, Term},
    sparql::{QueryResults, QuerySolution, SparqlEvaluator, Variable},
    store::Store,
};

use crate::{
    cellml::{Citation, CitationAuthor},
    error::RdfIndexerError,
    read::BASE_IRI,
};

fn join_iri(iri: &str) -> Result<NamedNode, IriParseError> {
    Ok(NamedNode::new_unchecked(
        IriRef::parse(BASE_IRI)?
            .resolve(iri)?
            .into_inner()
    ))
}

fn named_node(node: Option<(&'static str, &str)>) -> Result<Option<(&'static str, NamedNode)>, IriParseError> {
    node.map(|(name, iri)| Ok::<_, IriParseError>((name, join_iri(iri)?)))
        .transpose()
}

fn query_solutions<F, T>(
    store: &Store,
    query: &'static str,
    root_node: Option<(&'static str, impl Into<Term>)>,
    extractor: F,
) -> Result<Vec<T>, RdfIndexerError>
where
    F: Fn(QuerySolution) -> Option<T>,
{
    let mut result = Vec::new();
    let mut query = SparqlEvaluator::new().parse_query(query)?;

    if let Some((node_id, term)) = root_node {
        let var = Variable::new(node_id)
            .expect("specified static node_id must parse correctly");
        query = query.substitute_variable(var, term);
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
    Ok(query_solutions(
        store,
        query,
        named_node(root_node)?,
        format_solution(var_id, formatter, literal, iri),
    )?)
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
    Ok(query_solutions(
        store,
        r#"
        PREFIX bqs: <http://www.cellml.org/bqs/1.0#>
        PREFIX dc: <http://purl.org/dc/elements/1.1/>
        PREFIX dcterms: <http://purl.org/dc/terms/>

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
        named_node(node.map(|node| ("node", node)))?,
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
            if let Some(Term::Literal(literal)) = solution.get("volume") {
                citation.volume = Some(literal.value().to_string())
            }
            if let Some(Term::Literal(literal)) = solution.get("first_page") {
                citation.first_page = Some(literal.value().to_string())
            }
            if let Some(Term::Literal(literal)) = solution.get("last_page") {
                citation.last_page = Some(literal.value().to_string())
            }
            if let Some(Term::Literal(literal)) = solution.get("issued") {
                // TODO verify the value is in fact a date
                citation.issued = Some(literal.value().to_string())
            }
            if let Some(Term::NamedNode(node)) = solution.get("croot") {
                if let Ok(authors) = creators(store, node.clone()) {
                    citation.authors = authors
                }
            }
            if let Some(Term::BlankNode(node)) = solution.get("croot") {
                if let Ok(authors) = creators(store, node.clone()) {
                    citation.authors = authors
                }
            }
            Some(citation)
        },
    )?)
}

pub fn creators(store: &Store, node: impl Into<Term>) -> Result<Vec<CitationAuthor>, RdfIndexerError> {
    // first, gather all terms associated with vcards under the node of interest
    let term = node.into();

    let mut vcard_terms = Vec::new();
    vcard_terms.append(&mut query_solutions(
        store,
        r#"
	PREFIX bqs: <http://www.cellml.org/bqs/1.0#>
	PREFIX vCard: <http://www.w3.org/2001/vcard-rdf/3.0#>

	SELECT ?node ?vcnode
	WHERE {
	    ?node ?li ?creator .
	    ?creator bqs:Person ?person .
	    ?person vCard:N ?vcnode .
	}
	ORDER BY ?li
        "#,
        Some(("node", term.clone())),
        |solution| solution.get("vcnode").map(Clone::clone),
    )?);
    vcard_terms.append(&mut query_solutions(
        store,
        r#"
	PREFIX bqs: <http://www.cellml.org/bqs/1.0#>
	PREFIX vCard: <http://www.w3.org/2001/vcard-rdf/3.0#>

	SELECT ?node ?vcnode
	WHERE {
	    ?node ?li ?creator .
	    ?creator vCard:N ?vcnode .
	}
	ORDER BY ?li
        "#,
        Some(("node", term.clone())),
        |solution| solution.get("vcnode").map(Clone::clone),
    )?);

    let mut results = Vec::new();

    for vcard_term in vcard_terms.into_iter() {
        if let Some(mut author) = query_solutions(
            store,
            r#"
            PREFIX vCard: <http://www.w3.org/2001/vcard-rdf/3.0#>

            SELECT ?vcnode ?family ?given
            WHERE {
                ?vcnode vCard:Family ?family .
                OPTIONAL { ?vcnode vCard:Given ?given } .
            }
            ORDER BY ?li
            "#,
            Some(("vcnode", vcard_term.clone())),
            |solution| {
                let mut author = CitationAuthor::default();
                if let Some(Term::Literal(literal)) = solution.get("family") {
                    author.family = literal.value().to_string();
                }
                if let Some(Term::Literal(literal)) = solution.get("given") {
                    author.given = Some(literal.value().to_string());
                }
                Some(author)
            }
        )?.pop() {
            author.other = query_solutions(
                store,
                r#"
                PREFIX vCard: <http://www.w3.org/2001/vcard-rdf/3.0#>

                SELECT ?vcnode ?other
                WHERE {
                    ?vcnode vCard:Other ?other .
                }
                ORDER BY ?li
                "#,
                Some(("vcnode", vcard_term)),
                |solution| {
                    if let Some(Term::Literal(literal)) = solution.get("other") {
                        Some(literal.value().to_string())
                    } else {
                        None
                    }
                }
            )?;
            results.push(author);
        }
    }

    Ok(results)
}
