use serde::{Deserialize, Serialize};

pub mod query;
pub mod cmeta;

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

impl VCardInfo {
    pub fn fullname(&self) -> Option<String> {
        let mut result = None;
        if let Some(given) = &self.given {
            result.get_or_insert(given.to_string());
        }
        if let Some(family) = &self.family {
            result = Some(result.map(|x| format!("{x} {family}"))
                .unwrap_or_else(|| family.to_string()));
        }
        result
    }

    pub fn org(&self) -> Option<String> {
        let mut result = None;
        if let Some(orgunit) = &self.orgunit {
            result.get_or_insert(orgunit.to_string());
        }
        if let Some(orgname) = &self.orgname {
            result = Some(result.map(|x| format!("{x}, {orgname}"))
                .unwrap_or_else(|| orgname.to_string()));
        }
        result
    }
}
