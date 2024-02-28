use serde::{Deserialize, Serialize};
use crate::workspace::Workspace;

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
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FileInfo {
    pub size: u64,
    pub binary: bool,
    pub mime_type: String,
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
    /// The url of the remote location.
    pub location: String,
    /// The commit id of the interested target.
    pub commit: String,
    /// The path of the interested target.
    pub subpath: String,
    /// The original path that resolved this.
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ObjectInfo {
    FileInfo(FileInfo),
    TreeInfo(TreeInfo),
    CommitInfo(CommitInfo),
    LogInfo(LogInfo),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PathObjectInfo {
    FileInfo(FileInfo),
    TreeInfo(TreeInfo),
    RemoteInfo(RemoteInfo),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RepoResult {
    /// The workspace that this result was derived from.
    pub workspace: Workspace,
    /// The commit id that this result was derived from.
    pub commit: CommitInfo,
    /// The path to the target.
    pub path: String,
    /// The target is the resource identified at the path.
    pub target: PathObjectInfo,
}

/*
// This was the original result, but it lacked workspace and description
// so it wasn't too useful, but given that a repo is ultimately from a
// workspace so it make sense to see if the above, more detailed result
// type, is more useful.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathInfo {
    // TODO figure out if we need these back, as the "merged" version
    // was removed to keep the amount of objects simple.
    // TODO need to see if the end point really need this, or is there
    // a better way if leptos routing can fix this issue.
    // TODO determine if the more verbose RepoResult is just going to
    // be the better, although albeit, verbose way.
    // TODO comment/uncomment workspace_id/description field while all
    // related questions are being worked on.
    pub workspace_id: i64,
    pub description: Option<String>,
    pub commit: CommitInfo,
    pub path: String,
    pub object: ObjectInfo,
}
*/
