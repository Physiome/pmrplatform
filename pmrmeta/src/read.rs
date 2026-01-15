use std::io::Read;
use oxigraph::{
    model::Quad,
    store::Store,
};

use crate::{
    error::RdfIndexerError,
    xml::Xml,
};

pub fn quads_from_xml<R>(reader: R) -> Result<Vec<Quad>, RdfIndexerError>
where
    R: Read,
{
    let mut xml = Xml::new(reader)?;
    xml.to_quads()
}

pub fn xml_to_store<R>(reader: R) -> Result<Store, RdfIndexerError>
where
    R: Read,
{
    let results = quads_from_xml(reader)?;
    let store = Store::new()?;
    store.extend(results)?;
    Ok(store)
}
