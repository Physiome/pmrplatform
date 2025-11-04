use std::io::Read;
use oxigraph::{
    io::{RdfFormat, RdfParser},
    model::Term,
    sparql::{QueryResults, SparqlEvaluator},
    store::Store,
};
use xee_xpath::{
    context::StaticContextBuilder,
    Documents, Itemable, Queries, Query,
};

use crate::error::RdfIndexerError;

pub fn xml_to_store<R>(mut reader: R) -> Result<Store, RdfIndexerError>
where
    R: Read
{
    let mut input_xml = String::new();
    reader.read_to_string(&mut input_xml)?;
    let mut documents = Documents::new();
    let doc = documents.add_string_without_uri(&input_xml)?;
    let mut context_builder = StaticContextBuilder::default();
    context_builder.add_namespace("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#");
    let queries = Queries::new(context_builder);

    let sequence_query = queries.sequence("//rdf:RDF")?;
    let mut dynamic_context_builder = sequence_query.dynamic_context_builder(&mut documents);
    dynamic_context_builder.context_item(doc.to_item(&documents)?);
    let context = dynamic_context_builder.build();

    let sequence = sequence_query
        .execute_with_context(&mut documents, &context)?
        .flatten()?;

    let xot = documents.xot();

    let extracted = sequence.iter()
        .map(|item| item.display_representation(xot, &context))
        .collect::<Result<Vec<_>, _>>()?
        .join("");

    let results = RdfParser::from_format(RdfFormat::RdfXml)
        // .with_base_iri("urn:pmr:virtuoso:")
        .with_base_iri("urn:pmrplatform:oxigraph:")?
        .for_reader(extracted.as_bytes())
        .collect::<Result<Vec<_>, _>>()?;
    // println!("{results:?}");

    let store = Store::new()?;
    store.extend(results)?;
    Ok(store)
}

pub fn index<R>(reader: R) -> Result<Option<String>, RdfIndexerError>
where
    R: Read
{
    let store = xml_to_store(reader)?;

    if let QueryResults::Solutions(mut solutions) = SparqlEvaluator::new()
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
        // for solution in solutions {
        //     if let Ok(solution) = solution {
        //         println!("{:?}", solution.get("pmid"));
        //     }
        // }
        if let Some(Ok(solution)) = solutions.next() {
            if let Some(Term::Literal(literal)) = solution.get("pmid") {
                return Ok(Some(literal.to_string()));
            }
        }
    }
    Ok(None)
}
