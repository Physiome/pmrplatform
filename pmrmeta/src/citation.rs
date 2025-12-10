use std::io::Read;
use crate::{
    cellml::meta::query_pubmed_id,
    error::RdfIndexerError,
    read::xml_to_store,
};

pub fn index<R>(reader: R) -> Result<Vec<String>, RdfIndexerError>
where
    R: Read
{
    let store = xml_to_store(reader)?;
    query_pubmed_id(&store)
}
