use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
pub struct Citation {
    pub id: String,
    pub authors: Vec<CitationAuthor>,
    pub title: String,
    pub journal: Option<String>,
    pub volume: Option<String>,
    pub first_page: Option<String>,
    pub last_page: Option<String>,
    pub issued: Option<String>,
}

#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
pub struct CitationAuthor {
    pub family: String,
    pub given: Option<String>,
    pub other: Option<String>,
}

pub mod traits;
