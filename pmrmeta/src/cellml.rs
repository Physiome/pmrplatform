use serde::{Deserialize, Serialize};
pub mod query;

#[derive(Default, Debug, PartialEq, Deserialize, Serialize)]
pub struct CitationAuthor {
    family: String,
    given: Option<String>,
    other: Vec<String>,
}

#[derive(Default, Debug, PartialEq, Deserialize, Serialize)]
pub struct Citation {
    id: Option<String>,
    authors: Vec<CitationAuthor>,
    title: Option<String>,
    journal: Option<String>,
    volume: Option<String>,
    first_page: Option<String>,
    last_page: Option<String>,
    issued: Option<String>,
}
