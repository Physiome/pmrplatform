use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

use pmrbin::citation::index;

const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn load_test_data(name: &str) -> std::io::Result<Vec<u8>> {
    let data_path = PathBuf::from(CARGO_MANIFEST_DIR)
        .join("tests")
        .join("data")
        .join(name);
    let mut fd = BufReader::new(File::open(data_path)?);
    let mut result = Vec::new();
    fd.read_to_end(&mut result)?;
    Ok(result)
}

#[test]
fn index1() -> anyhow::Result<()> {
    let input = load_test_data("beeler_reuter_model_1977.cellml")?;
    let mut result = index(&input[..])?;
    result.sort();
    assert_eq!(result, &["pmid:874889"]);
    Ok(())
}

#[test]
fn index2() -> anyhow::Result<()> {
    let input = load_test_data("adrian_chandler_hodgkin_1970_version01.cellml")?;
    let mut result = index(&input[..])?;
    result.sort();
    assert_eq!(result, &["pmid:4778131", "pmid:5499787"]);
    Ok(())
}
