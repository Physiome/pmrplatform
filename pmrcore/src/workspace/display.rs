use chrono::{LocalResult, TimeZone, Utc};
use std::fmt::{
    Display,
    Formatter,
    Result,
};
use crate::workspace::*;

impl Display for Workspace {
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

impl Display for WorkspaceAlias {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{} - {}",
            &self.workspace_id,
            &self.alias,
        )
    }
}

impl Display for WorkspaceSync {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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
            WorkspaceSyncStatus::from(self.status),
        )
    }
}

impl Display for WorkspaceTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{} - {}",
            &self.commit_id,
            &self.name,
        )
    }
}
