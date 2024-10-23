//! Generated Policy
//!
//! The structs provided by this module represents policies generated
//! for consumption by some security enforcer, and is not meant to be
//! persisted in some datastore.

use serde::{Deserialize, Serialize};
use crate::ac::role::Role;

/// Grants, roles and permissions associated with the given resource
/// to be passed into the security enforcer as a complete policy.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Policy {
    pub resource: String,
    pub user_roles: Vec<UserRole>,
    pub res_grants: Vec<ResGrant>,
    pub role_permits: Vec<RolePermit>,
}

/// A resource grant - the agent will have the stated role at the given
/// resource.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct ResGrant {
    // this may feel redundant later, but this line signifies the exact
    // res this was granted for, which may be at a higher level.
    pub res: String,
    pub agent: Option<String>,
    pub role: Role,
}

/// This represents the action the role is given the permit for.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct RolePermit {
    pub role: Role,
    pub action: String,
}

// FIXME should this be AgentRole instead?
// or are we making the system easier to not allow management of anonymous?
/// Represents the role granted to the user for the system.  Roles
/// granted this way is only applicable for resources at some
/// appropriate state.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct UserRole {
    pub user: String,
    pub role: Role,
}

mod impls;
