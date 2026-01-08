use oxigraph::store::Store;
use pmrmeta::{
    cellml::query,
    read::{
        quads_from_xml,
        xml_to_store,
    },
};

mod utils;

#[test]
fn keywords1() -> anyhow::Result<()> {
    let store = xml_to_store(&utils::load_test_data("beeler_reuter_model_1977.cellml")?[..])?;
    let mut result = query::keywords(&store)?;
    result.sort();
    assert_eq!(result, &["electrophysiological", "ventricular myocyte"]);
    Ok(())
}

#[test]
fn keywords2() -> anyhow::Result<()> {
    let store = xml_to_store(&utils::load_test_data("adrian_chandler_hodgkin_1970_version01.cellml")?[..])?;
    let mut result = query::keywords(&store)?;
    result.sort();
    assert_eq!(result, &["electrophysiology", "skeletal muscle"]);
    Ok(())
}

// This really is a combination test to show how merging multiple files into
// a single store might work (it doesn't work well to distinguish data source,
// but it does work).
#[test]
fn pubmed_id() -> anyhow::Result<()> {
    let store = Store::new()?;
    store.extend(quads_from_xml(
        &utils::load_test_data("beeler_reuter_model_1977.cellml")?[..]
    )?)?;
    store.extend(quads_from_xml(
        &utils::load_test_data("adrian_chandler_hodgkin_1970_version01.cellml")?[..]
    )?)?;
    let mut result = query::pubmed_id(&store)?;
    result.sort();
    assert_eq!(result, &["pmid:4778131", "pmid:5499787", "pmid:874889"]);
    Ok(())
}

#[test]
fn title() -> anyhow::Result<()> {
    let store = xml_to_store(&utils::load_test_data("baylor_hollingworth_chandler_2002_a.cellml")?[..])?;
    let result = query::dc_title(&store, Some(""))?;
    assert_eq!(result, &["Comparison of Simulated and Measured Calcium Sparks in Intact Skeletal Muscle Fibers of the Frog (Reaction A)"]);
    Ok(())
}

#[test]
fn license() -> anyhow::Result<()> {
    let store = xml_to_store(&utils::load_test_data("example_model.cellml")?[..])?;
    let result = query::license(&store)?;
    assert_eq!(result.as_deref(), Some("http://example.com/license"));
    Ok(())
}
