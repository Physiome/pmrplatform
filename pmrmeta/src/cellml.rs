use serde::{Deserialize, Serialize};
pub mod query;

#[derive(Default, Debug, PartialEq, Deserialize, Serialize)]
pub struct CitationAuthor {
    pub family: String,
    pub given: Option<String>,
    pub other: Vec<String>,
}

#[derive(Default, Debug, PartialEq, Deserialize, Serialize)]
pub struct Citation {
    pub id: Option<String>,
    pub authors: Vec<CitationAuthor>,
    pub title: Option<String>,
    pub journal: Option<String>,
    pub volume: Option<String>,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
    pub issued: Option<String>,
}

#[derive(Default, Debug, PartialEq, Deserialize, Serialize)]
pub struct VCardInfo {
    pub family: Option<String>,
    pub given: Option<String>,
    pub orgname: Option<String>,
    pub orgunit: Option<String>,
}
