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
}
