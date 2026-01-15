use std::io::Read;
use oxigraph::{
    io::{RdfFormat, RdfParser},
    model::Quad,
};
use xee_xpath::{
    context::StaticContextBuilder,
    Documents, DocumentHandle, Item, Itemable, Queries, Query,
};

use crate::error::{RdfIndexerError, XeeError};

pub static BASE_IRI: &str = "urn:pmrplatform:oxigraph:";

// Provide an abstraction that doesn't separate out the main document handle from the
// collection of documents to make a less cumbersome API.
pub(crate) struct Xml {
    documents: Documents,
    handle: DocumentHandle,
}

impl Xml {
    pub fn new<R>(mut reader: R) -> Result<Self, XeeError>
    where
        R: Read,
    {
        let mut input_xml = String::new();
        reader.read_to_string(&mut input_xml)?;
        let mut documents = Documents::new();
        let handle = documents.add_string_without_uri(&input_xml)?;
        Ok(Self { documents, handle })
    }

    pub fn xpath(&mut self, s: &str) -> Result<Vec<String>, XeeError> {
        let mut context_builder = StaticContextBuilder::default();
        context_builder.add_namespace("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#");
        context_builder.add_namespace("cmeta", "http://www.cellml.org/metadata/1.0#");
        let queries = Queries::new(context_builder);
        let sequence_query = queries.sequence(s)?;
        let mut dynamic_context_builder = sequence_query.dynamic_context_builder(&mut self.documents);
        dynamic_context_builder.context_item(self.handle.to_item(&self.documents)?);
        let context = dynamic_context_builder.build();
        let sequence = sequence_query
            .execute_with_context(&mut self.documents, &context)?
            .flatten()?;
        let xot = self.documents.xot();
        Ok(sequence.iter()
            .map(|item| match item {
                Item::Atomic(atomic) => atomic.to_string(),
                _ => item.display_representation(xot, &context),
            })
            .collect::<Result<Vec<_>, _>>()?)
    }

    pub fn to_quads(&mut self) -> Result<Vec<Quad>, RdfIndexerError> {
        let extracted = self.xpath("//rdf:RDF")?.join("");
        Ok(RdfParser::from_format(RdfFormat::RdfXml)
            // .with_base_iri("urn:pmr:virtuoso:")
            .with_base_iri(BASE_IRI)?
            .for_reader(extracted.as_bytes())
            .collect::<Result<Vec<_>, _>>()?)
    }
}
