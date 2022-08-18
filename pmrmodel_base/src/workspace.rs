use serde::{Deserialize, Serialize};
use std::fmt::{
    Display,
    Formatter,
    Result,
};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct WorkspaceRecord {
    pub id: i64,
    pub url: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct JsonWorkspaceRecords {
    pub workspaces: Vec<WorkspaceRecord>
}

impl Display for WorkspaceRecord {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{} - {} - {}",
            self.id,
            &self.url,
            match &self.description {
                Some(v) => v,
                None => "<empty>",
            },
        )
    }
}
