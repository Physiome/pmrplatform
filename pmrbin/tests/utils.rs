use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};

const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

pub fn load_test_data(name: &str) -> std::io::Result<Vec<u8>> {
    let data_path = PathBuf::from(CARGO_MANIFEST_DIR)
        .join("tests")
        .join("data")
        .join(name);
    let mut fd = BufReader::new(File::open(data_path)?);
    let mut result = Vec::new();
    fd.read_to_end(&mut result)?;
    Ok(result)
}
