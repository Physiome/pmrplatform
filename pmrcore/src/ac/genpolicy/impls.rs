use enumset::EnumSet;
use crate::ac::role::Roles;
use super::*;

impl Policy {
    pub fn to_roles(&self) -> Roles {
        let mut results = Roles(
            self.agent_roles
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
}

impl From<(Option<String>, Role)> for AgentRole {
    fn from((agent, role): (Option<String>, Role)) -> Self {
        Self { agent, role }
    }
}
