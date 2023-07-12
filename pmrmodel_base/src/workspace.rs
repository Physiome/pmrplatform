use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Workspace {
    pub id: i64,
    pub url: String,
    pub superceded_by_id: Option<i64>,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub created_ts: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Workspaces(Vec<Workspace>);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct WorkspaceAlias {
    pub id: i64,
    pub workspace_id: i64,
    pub alias: String,
    pub created_ts: i64,
}

#[derive(Debug, PartialEq, FromPrimitive)]
#[repr(i64)]
pub enum WorkspaceSyncStatus {
    Completed,
    Running,
    Error,
    #[num_enum(default)]
    Unknown = -1,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct WorkspaceSync {
    pub id: i64,
    pub workspace_id: i64,
    pub start: i64,
    pub end: Option<i64>,
    pub status: i64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct WorkspaceTag {
    pub id: i64,
    pub workspace_id: i64,
    pub name: String,
    pub commit_id: String,
}

mod impls {
    use std::ops::{
        Deref,
        DerefMut,
    };
    use crate::workspace::*;

    impl From<Vec<Workspace>> for Workspaces {
        fn from(args: Vec<Workspace>) -> Self {
            Self(args)
        }
    }

    impl<const N: usize> From<[Workspace; N]> for Workspaces {
        fn from(args: [Workspace; N]) -> Self {
            Self(args.into())
        }
    }

    impl Deref for Workspaces {
        type Target = Vec<Workspace>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl DerefMut for Workspaces {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}

#[cfg(feature = "display")]
mod display {
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
}
