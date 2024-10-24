use enumset::EnumSet;
use crate::ac::role::Roles;
use super::*;

impl Policy {
    pub fn to_roles(&self) -> Roles {
        let mut results = Roles(
            self.user_roles
                .iter()
                .map(|v| v.role)
                .collect()
        );
        results.0 |= self.res_grants
            .iter()
            .map(|v| v.role)
            .collect::<EnumSet<_>>();
        results
    }

    pub fn to_action_role_permit_map(&self) -> ActionRolePermitMap {
        ActionRolePermitMap::from_iter(self.role_permits.clone().into_iter())
    }
}

impl From<Policy> for PolicyEnforcer {
    fn from(policy: Policy) -> Self {
        Self {
            roles: policy.to_roles(),
            permit_map: policy.to_action_role_permit_map(),
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
                m.entry(action.clone())
                    .or_default()
                    .0 |= role;
                m
            })
        )
    }
}

impl PolicyEnforcer {
    pub fn enforce(&self, action: &str) -> bool {
        self.permit_map.0
            .get(action)
            .map(|roles| roles.0 & self.roles.0 != EnumSet::empty())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn policy_enforcer() -> anyhow::Result<()> {
        let policy: Policy = serde_json::from_str(r#"{
            "resource": "/item/1",
            "user_roles": [
                {
                    "user": "alice",
                    "role": "Reader"
                }
            ],
            "res_grants": [
                {"res": "/item/1", "agent": "alice", "role": "Owner"}
            ],
            "role_permits": [
                {"role": "Owner", "action": "editor_view"},
                {"role": "Owner", "action": "editor_edit"},
                {"role": "Reader", "action": ""}
            ]
        }"#)?;
        let enforcer: PolicyEnforcer = policy.into();
        assert!(enforcer.enforce("editor_view"));
        assert!(enforcer.enforce("editor_edit"));
        assert!(!enforcer.enforce("grant_edit"));
        assert!(enforcer.enforce(""));
        Ok(())
    }
}
