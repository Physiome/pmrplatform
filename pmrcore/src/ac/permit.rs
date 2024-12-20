use serde::{Deserialize, Serialize};

use crate::ac::role::Role;

/// Resource grant
///
/// Represents the access granted to a resource.  The grant associates
/// the user and the role for the resource.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ResGrant {
    pub id: i64,
    pub res: String,
    pub user_id: i64,
    pub user_name: String,
    pub role: Role,
}

/// Workflow policy
///
/// For each workflow state there may be multiple roles associated with
/// the different actions.  This struct will only capture a single such
/// record.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct WorkflowPolicy {
    pub id: i64,
    pub state: String,
    pub role: String,
    pub action: String,
}
