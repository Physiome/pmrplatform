use oxigraph::store::Store;
use pmrmeta::{
    cellml::{
        Citation,
        query,
    },
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

#[test]
fn citation_named_nodes() -> anyhow::Result<()> {
    let store = xml_to_store(&utils::load_test_data("adrian_chandler_hodgkin_1970_version01.cellml")?[..])?;
    let result = query::citation(&store, None)?;
    let expected = serde_json::from_str::<Vec<Citation>>(r#"[{
        "id": "urn:miriam:pubmed:4778131",
        "authors": [{
            "family": "Adrian",
            "given": "R",
            "other": ["H"]
        }, {
            "family": "Peachey",
            "given": "L",
            "other": ["D"]
        }],
        "title": "Reconstruction of the Action Potential of Frog Sartorius Muscle",
        "journal": "Journal of Physiology",
        "volume": "235",
        "first_page": "103",
        "last_page": "131",
        "issued": null
    }, {
        "id": "urn:miriam:pubmed:5499787",
        "authors": [{
            "family": "Adrian",
            "given": "R",
            "other": ["H"]
        }, {
            "family": "Chandler",
            "given": "W",
            "other": ["K"]
        }, {
            "family": "Hodgkin",
            "given": "A",
            "other": ["L"]
        }],
        "title": "Voltage Clamp Experiments in Striated Muscle Fibres",
        "journal": "Journal of Physiology",
        "volume": "208",
        "first_page": "607",
        "last_page": "644",
        "issued": null
    }]"#)?;
    assert_eq!(result, expected);
    Ok(())
}

#[test]
fn citation_blank_nodes() -> anyhow::Result<()> {
    let store = xml_to_store(&utils::load_test_data("detailed_citation.cellml")?[..])?;
    let result = query::citation(&store, None)?;
    let expected = serde_json::from_str::<Vec<Citation>>(r#"[{
	"id": null,
	"authors": [{
	    "family": "Author",
	    "given": "Main",
	    "other": []
	}, {
	    "family": "Family2",
	    "given": "Hello",
	    "other": []
	}, {
	    "family": "Family3",
	    "given": "User",
	    "other": []
	}],
	"title": "Journal article title to the model",
	"journal": "Some Journal",
	"volume": "11",
	"first_page": "1234",
	"last_page": "1236",
	"issued": null
    }]"#)?;
    assert_eq!(result, expected);
    Ok(())
}
