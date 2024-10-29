use pmrcore::ac::{
    genpolicy::Policy,
    role::Roles,
};
use std::collections::HashMap;

pub struct ActionRolePermitMap(HashMap<String, Roles>);

/// A simplified enforcer that provides methods that will do a direct
/// check of the included roles against the actions desired.  It assumes
/// the policy is fully contained and constrained for the originating
/// `Policy` for the user and resource that should have produced it.
pub struct PolicyEnforcer {
    policy: Policy,
    roles: Roles,
    permit_map: ActionRolePermitMap,
}

mod impls;
