use std::io::Read;
use oxigraph::{
    io::{RdfFormat, RdfParser},
    model::Quad,
};
use xee_xpath::{
    context::StaticContextBuilder,
    Documents, DocumentHandle, Item, Itemable, Queries, Query,
};

use crate::error::{RdfIndexerError, XeeError, XrustError};

pub static BASE_IRI: &str = "urn:pmrplatform:oxigraph:";
const CELLML_TMPDOC_TO_HTML: &str = include_str!("data/cellml_tmpdoc-to-html.xslt");

// Provide an abstraction that doesn't separate out the main document handle from the
// collection of documents to make a less cumbersome API.
pub struct Xml {
    documents: Documents,
    handle: DocumentHandle,
    input_xml: String,
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
        Ok(Self { documents, handle, input_xml })
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


mod xslt {
    use xrust::{
        item::{Item, Node, NodeType, Sequence, SequenceTrait},
        trees::smite::RNode,
        parser::{ParseError, xml::parse},
        transform::{
            context::{StaticContext, StaticContextBuilder},
            Transform,
        },
        xdmerror::{Error, ErrorKind},
        xslt::from_document,
    };

    use super::*;

    impl Xml {
        pub fn xslt(&self, src: &str) -> Result<String, XrustError> {
            // hopefully xee will support xslt eventually; there are imports that conflict so
            // only have it here.
            let doc = Item::Node(parse(
                RNode::new_document(),
                &self.input_xml,
                Some(|_: &_| Err(ParseError::MissingNameSpace)),
            )?);
            let xslt = parse(
                RNode::new_document(),
                CELLML_TMPDOC_TO_HTML,
                Some(|_: &_| Err(ParseError::MissingNameSpace)),
            )?;
            let mut static_context = StaticContextBuilder::new()
                .message(|_| Ok(()))
                .fetcher(|_| Err(Error::new(ErrorKind::NotImplemented, "not implemented")))
                .parser(|_| Err(Error::new(ErrorKind::NotImplemented, "not implemented")))
                .build();
            let mut ctx = from_document(
                xslt,
                None,
                |s: &str| parse(RNode::new_document(), s, Some(|_: &_| Err(ParseError::MissingNameSpace))),
                |_| Ok(String::new()),
            )?;

            ctx.context(vec![doc], 0);
            ctx.result_document(RNode::new_document());

            let out = ctx.evaluate(&mut static_context)?;

            Ok(out.to_xml())
        }
    }
}
