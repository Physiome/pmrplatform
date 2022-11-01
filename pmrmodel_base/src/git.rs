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
pub struct FileInfo {
    pub size: u64,
    pub binary: bool,
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TreeInfo {
    pub filecount: u64,
    pub entries: Vec<TreeEntryInfo>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommitInfo {
    pub commit_id: String,
    pub author: String,
    pub committer: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogInfo {
    // TODO fields about start, next, pagination?
    pub entries: Vec<LogEntryInfo>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RemoteInfo {
    pub location: String,
    pub commit: String,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ObjectInfo {
    FileInfo(FileInfo),
    TreeInfo(TreeInfo),
    CommitInfo(CommitInfo),
    LogInfo(LogInfo),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PathObject {
    FileInfo(FileInfo),
    TreeInfo(TreeInfo),
    RemoteInfo(RemoteInfo),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathInfo {
    pub commit: CommitInfo,
    pub path: String,
    pub object: Option<PathObject>,
}
