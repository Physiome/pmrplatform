use serde::{Deserialize, Serialize};
use crate::ac::user::User;

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

/// A grant that will be passed onto the enforcer.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Grant {
    // this may feel redundant later, but this line signifies the exact
    // res this was granted for, which may be at a higher level.
    pub res: String,
    pub user: String,
    pub role: String,
}

/// A policy entry that will be passed onto the enforcer.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Policy {
    pub role: String,
    pub endpoint_group: String,
    pub method: String,
}

/// Grants and resources associated with the resource ready to be passed
/// into the enforcer.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ResourcePolicy {
    pub resource: String,
    pub grants: Vec<Grant>,
    pub policies: Vec<Policy>,
}
