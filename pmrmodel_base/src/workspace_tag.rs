use std::fmt;

pub struct WorkspaceTagRecord {
    pub id: i64,
    pub workspace_id: i64,
    pub name: String,
    pub commit_id: String,
}

impl std::fmt::Display for WorkspaceTagRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} - {}",
            &self.commit_id,
            &self.name,
        )
    }
}
