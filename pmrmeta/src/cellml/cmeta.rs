use std::io::Read;

use oxigraph::store::Store;
use pmrcore::citation::Citation;
use serde::{Deserialize, Serialize};

use crate::{
    cellml::{VCardInfo, query},
    error::RdfIndexerError,
    xml::Xml,
};

pub struct Cmeta {
    store: Store,
    root_cmetaid: Option<String>,
    cmetaids: Vec<String>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct Pmr2Cmeta {
    #[serde(rename = "citation_bibliographicCitation")]
    pub citation_bibliographic_citation: Option<String>,
    // names (last, first, second)
    pub citation_authors: Option<Vec<(String, String, String)>>,
    pub citation_title: Option<String>,
    pub citation_id: Option<String>,
    pub citation_issued: Option<String>,
    pub model_author: Option<String>,
    pub model_author_org: Option<String>,
    pub model_title: Option<String>,
    // keyword (location, identifier)
    pub keywords: Option<Vec<(String, String)>>,

    pub citations: Vec<Citation>,
}

impl Cmeta {
    pub fn new<R>(reader: R) -> Result<Self, RdfIndexerError>
    where
        R: Read,
    {
        let mut xml = Xml::new(reader)?;
        let root_cmetaid = xml.xpath("/*/@cmeta:id/string()")?.pop();
        let cmetaids = xml.xpath("//@cmeta:id/string()")?;
        let store = Store::new()?;
        store.extend(xml.to_quads()?)?;
        Ok(Self { store, root_cmetaid, cmetaids })
    }

    pub fn keywords(&self) -> Result<Vec<String>, RdfIndexerError> {
        query::keywords(&self.store)
    }

    pub fn contextual_keywords(&self) -> Result<Vec<(String, String)>, RdfIndexerError> {
        query::contextual_keywords(&self.store)
    }

    pub fn dc_title(&self, node: Option<&str>) -> Result<Vec<String>, RdfIndexerError> {
        query::dc_title(&self.store, node)
    }

    pub fn license(&self) -> Result<Option<String>, RdfIndexerError> {
        query::license(&self.store)
    }

    pub fn citation(&self, node: Option<&str>) -> Result<Vec<Citation>, RdfIndexerError> {
        query::citation(&self.store, node)
    }

    pub fn dc_vcard_info(&self, node: Option<&str>) -> Result<Vec<VCardInfo>, RdfIndexerError> {
        query::dc_vcard_info(&self.store, node)
    }

    pub fn root_cmetaid(&self) -> Option<&str> {
        self.root_cmetaid.as_deref()
    }

    pub fn cmetaids(&self) -> &[String] {
        &self.cmetaids
    }
}
