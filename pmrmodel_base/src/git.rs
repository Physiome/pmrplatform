use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TreeEntryInfo {
    pub filemode: String,
    pub kind: String,
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogEntryInfo {
    pub commit_id: String,
    pub author: String,
    pub committer: String,
    pub commit_timestamp: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ObjectInfo {
    FileInfo {
        size: u64,
        binary: bool,
    },
    TreeInfo {
        filecount: u64,
        entries: Vec<TreeEntryInfo>,
    },
    CommitInfo {
        commit_id: String,
        author: String,
        committer: String,
    },
    LogInfo {
        // TODO fields about start, next, pagination?
        entries: Vec<LogEntryInfo>,
    },
}
