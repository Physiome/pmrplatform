use pmrcore::ac::{
    agent::Agent,
    genpolicy::{
        Policy,
        RolePermit,
    },
    role::Roles,
    traits::Enforcer,
};
use std::collections::HashMap;

use crate::error::Error;
use super::*;

impl From<Policy> for PolicyEnforcer {
    fn from(policy: Policy) -> Self {
        Self {
            roles: policy.to_roles(),
            permit_map: ActionRolePermitMap::from_iter(
                policy.role_permits.clone().into_iter()
            ),
            policy
        }
    }
}

impl From<PolicyEnforcer> for Policy {
    fn from(enforcer: PolicyEnforcer) -> Self {
        enforcer.policy
    }
}

impl FromIterator<RolePermit> for ActionRolePermitMap {
    fn from_iter<I: IntoIterator<Item=RolePermit>>(iter: I) -> Self {
        Self(iter.into_iter()
            .fold(HashMap::new(), |mut m, RolePermit { action, role }| {
                *m.entry(action.clone())
                    .or_default() |= role;
                m
            })
        )
    }
}

impl Enforcer for PolicyEnforcer {
    type Error = Error;

    fn enforce(&self, agent: &Agent, res: &str, action: &str) -> Result<bool, Self::Error> {
        Ok(
            &self.policy.agent == agent &&
            self.policy.resource == res &&
            self.permit_map.0
                .get(action)
                .map(|roles| *roles & self.roles != Roles::empty())
                .unwrap_or_else(|| self.permit_map.0
                    .get("*")
                    .map(|roles| *roles & self.roles != Roles::empty())
                    .unwrap_or(false)
                )
        )
    }
}

#[cfg(test)]
mod test {
    use pmrcore::ac::user::User;
    use super::*;

    #[test]
    fn policy_enforcer() -> anyhow::Result<()> {
        let agent: Agent = User {
            id: 1,
            name: "alice".to_owned(),
            created_ts: 123456789,
        }.into();
        // the res: mismatched line is erroneously included.
        let policy: Policy = serde_json::from_str(r#"{
            "agent": {
                "User": {
                    "id": 1,
                    "name": "alice",
                    "created_ts": 123456789
                }
            },
            "resource": "/item/1",
            "user_roles": [
                {
                    "user": "alice",
                    "role": "Reader"
                }
            ],
            "res_grants": [
                {"res": "/mismached", "agent": "alice", "role": "Owner"},
                {"res": "/item/1", "agent": "alice", "role": "Owner"}
            ],
            "role_permits": [
                {"role": "Owner", "action": "editor_view"},
                {"role": "Owner", "action": "editor_edit"},
                {"role": "Reader", "action": ""}
            ]
        }"#)?;
        let enforcer: PolicyEnforcer = policy.into();
        assert!(enforcer.enforce(&agent, "/item/1", "editor_view")?);
        assert!(enforcer.enforce(&agent, "/item/1", "editor_edit")?);
        assert!(!enforcer.enforce(&agent, "/item/1", "grant_edit")?);
        assert!(enforcer.enforce(&agent, "/item/1", "")?);
        // mismatched agent
        assert!(!enforcer.enforce(&Agent::Anonymous, "/item/1", "")?);
        // mismatched resource
        assert!(!enforcer.enforce(&agent, "/mismatched", "")?);
        Ok(())
    }
}
