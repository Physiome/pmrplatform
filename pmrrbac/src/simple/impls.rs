use pmrcore::ac::{
    agent::Agent,
    genpolicy::{
        Policy,
        RolePermit,
    },
    role::Roles,
    traits::{
        Enforcer,
        GenpolEnforcer,
    },
};
use std::collections::HashMap;

use crate::error::Error;
use super::*;

impl PolicyEnforcer {
    pub fn policy(&self) -> &Policy {
        &self.policy
    }
}

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
            (
                self.permit_map.0
                    .get(action)
                    .map(|roles| *roles & self.roles != Roles::empty())
                    .unwrap_or(false) ||
                self.permit_map.0
                    .get("*")
                    .map(|roles| *roles & self.roles != Roles::empty())
                    .unwrap_or(false)
            )
        )
    }
}

impl GenpolEnforcer for PolicyEnforcer {
    type Error = Error;

    fn enforce(&self, action: &str) -> Result<bool, Self::Error> {
        Ok(
            self.permit_map.0
                .get(action)
                .map(|roles| *roles & self.roles != Roles::empty())
                .unwrap_or(false) ||
            self.permit_map.0
                .get("*")
                .map(|roles| *roles & self.roles != Roles::empty())
                .unwrap_or(false)
        )
    }
}

#[cfg(test)]
mod test {
    use pmrcore::ac::{
        agent::Agent,
        genpolicy::Policy,
        user::User,
    };

    use crate::Builder;
    use super::PolicyEnforcer;

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
            "agent_roles": [
                {
                    "agent": "alice",
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
        {
            use pmrcore::ac::traits::Enforcer;
            assert!(enforcer.enforce(&agent, "/item/1", "editor_view")?);
            assert!(enforcer.enforce(&agent, "/item/1", "editor_edit")?);
            assert!(!enforcer.enforce(&agent, "/item/1", "grant_edit")?);
            assert!(enforcer.enforce(&agent, "/item/1", "")?);
            // mismatched agent
            assert!(!enforcer.enforce(&Agent::Anonymous, "/item/1", "")?);
            // mismatched resource
            assert!(!enforcer.enforce(&agent, "/mismatched", "")?);
        }
        {
            use pmrcore::ac::traits::GenpolEnforcer;
            assert!(enforcer.enforce("editor_view")?);
            assert!(enforcer.enforce("editor_edit")?);
            assert!(!enforcer.enforce("grant_edit")?);
            assert!(enforcer.enforce("")?);
        }
        Ok(())
    }

    #[tokio::test]
    async fn builder_anonymous_reader() -> anyhow::Result<()> {
        let policy: Policy = serde_json::from_str(r#"{
            "agent": "Anonymous",
            "resource": "/item/1",
            "agent_roles": [],
            "res_grants": [],
            "role_permits": [
                {"role": "Reader", "action": ""}
            ]
        }"#)?;
        let builder = Builder::new().anonymous_reader(true);
        let enforcer = builder.build_with_policy(policy).await?;
        assert!(!enforcer.enforce("editor_view")?);
        assert!(!enforcer.enforce("grant_edit")?);
        assert!(enforcer.enforce("")?);
        Ok(())
    }

    #[tokio::test]
    async fn default_policy_enforcer() -> anyhow::Result<()> {
        let agent = Agent::default();
        let policy = Policy::default();
        let enforcer = PolicyEnforcer::from(policy);
        // enforcement using all default values should still fail.
        {
            use pmrcore::ac::traits::Enforcer;
            assert!(!enforcer.enforce(&agent, "", "")?);
        }
        {
            use pmrcore::ac::traits::GenpolEnforcer;
            assert!(!enforcer.enforce("")?);
        }
        Ok(())
    }
}
