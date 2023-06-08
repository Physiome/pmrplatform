use chrono::{LocalResult, TimeZone, Utc};
use enum_primitive::FromPrimitive;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
pub enum WorkspaceSyncStatus {
    Completed,
    Running,
    Error,
    Unknown = -1,
}
}

pub struct WorkspaceSyncRecord {
    pub id: i64,
    pub workspace_id: i64,
    pub start: i64,
    pub end: Option<i64>,
    pub status: i64,
}

impl std::fmt::Display for WorkspaceSyncRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} - {:?}",
            match Utc.timestamp_opt(self.start, 0) {
                LocalResult::Single(v) => v.to_rfc3339(),
                _ => "<invalid>".to_string(),
            },
            match self.end {
                Some(v) => match Utc.timestamp_opt(v, 0) {
                    LocalResult::Single(v) => v.to_rfc3339(),
                    _ => "<invalid>".to_string(),
                },
                None => "<nil>".to_string(),
            },
            WorkspaceSyncStatus::from_i64(self.status).unwrap_or(WorkspaceSyncStatus::Unknown),
        )
    }
}
