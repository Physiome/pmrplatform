pub mod query;

#[derive(Default)]
pub struct Citation {
    // names (last, first, second)
    id: Option<String>,
    authors: Vec<(String, String, String)>,
    title: Option<String>,
    journal: Option<String>,
    volume: Option<String>,
    // citation_issued: Option<String>,
}
