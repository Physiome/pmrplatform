use serde::{Deserialize, Serialize};

/// Resource grant
///
/// Represents the access granted to a resource.  The grant associates
/// the user and the role for the resource.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ResGrant {
    pub id: i64,
    pub res: String,
    pub user_id: i64,
    pub role: String,
}

/// Workfolow policy
///
/// For each workflow state there may be multiple roles associated with
/// the different endpoint groups and HTTP methods.  This struct will
/// only capture a single such record.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct WorkflowPolicy {
    pub id: i64,
    pub state: String,
    pub role: String,
    pub endpoint_group: String,
    pub method: String,
}
