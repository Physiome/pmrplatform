use pmrmeta::citation::index;
mod utils;

#[test]
fn index1() -> anyhow::Result<()> {
    let input = utils::load_test_data("beeler_reuter_model_1977.cellml")?;
    let mut result = index(&input[..])?;
    result.sort();
    assert_eq!(result, &["pmid:874889"]);
    Ok(())
}

#[test]
fn index2() -> anyhow::Result<()> {
    let input = utils::load_test_data("adrian_chandler_hodgkin_1970_version01.cellml")?;
    let mut result = index(&input[..])?;
    result.sort();
    assert_eq!(result, &["pmid:4778131", "pmid:5499787"]);
    Ok(())
}
