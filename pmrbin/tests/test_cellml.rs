use oxigraph::store::Store;
use pmrbin::{
    cellml::meta::{
        query_keywords,
        query_pubmed_id,
    },
    read::quads_from_xml,
};
mod utils;

#[test]
fn pubmed_id() -> anyhow::Result<()> {
    let store = Store::new()?;
    store.extend(quads_from_xml(
        &utils::load_test_data("beeler_reuter_model_1977.cellml")?[..]
    )?)?;
    store.extend(quads_from_xml(
        &utils::load_test_data("adrian_chandler_hodgkin_1970_version01.cellml")?[..]
    )?)?;
    let mut result = query_pubmed_id(&store)?;
    result.sort();
    assert_eq!(result, &["pmid:4778131", "pmid:5499787", "pmid:874889"]);
    Ok(())
}
