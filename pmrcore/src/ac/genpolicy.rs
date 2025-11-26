//! Generated Policy
//!
//! The structs provided by this module represents policies generated
//! for consumption by some security enforcer, and is not meant to be
//! persisted in some datastore.

use serde::{Deserialize, Serialize};
use crate::ac::agent::Agent;
use crate::ac::role::Role;

/// Grants, roles and permissions associated with the given resource
/// to be passed into the security enforcer as a complete policy.
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Policy {
    pub agent: Agent,
    pub resource: String,
    pub agent_roles: Vec<AgentRole>,
    pub res_grants: Vec<ResGrant>,
    pub role_permits: Vec<RolePermit>,
}

/// A resource grant - the agent will have the stated role at the given
/// resource.
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct ResGrant {
    // this may feel redundant later, but this line signifies the exact
    // res this was granted for, which may be at a higher level.
    pub res: String,
    pub agent: Option<String>,
    pub role: Role,
}

/// This represents the action the role is given the permit for.
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct RolePermit {
    pub role: Role,
    pub action: String,
}

/// Represents the role granted to the agent for the system.  Roles
/// granted this way is only applicable for resources when it is at the
/// appropriate state.
#[cfg_attr(feature="utoipa", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AgentRole {
    pub agent: Option<String>,
    pub role: Role,
}

mod impls;
