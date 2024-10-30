use pmrcore::ac::{
    genpolicy::Policy,
    role::Role,
};
use crate::{
    Enforcer,
    error::Error,
    simple::PolicyEnforcer,
};
#[cfg(feature = "casbin")]
use crate::casbin::{
    CasbinBuilder,
    CasbinEnforcer,
};

#[derive(Clone, Debug, Default)]
pub(crate) enum Kind {
    #[default]
    Policy,
    #[cfg(feature = "casbin")]
    Casbin(CasbinBuilder),
}

/// Builds a role-based access controller (RBAC) for PMR.
///
/// Methods can be chained in order to set the configuration values.
/// The `Enforcer` is constructed by calling [`build`].
///
/// New instances of the builder can be obtained via `Builder::default`
/// or `Builder::new`.  The former provides nothing while the latter
/// provides the default policy.
#[derive(Clone, Debug, Default)]
pub struct Builder {
    pub(crate) anonymous_reader: bool,
    pub(crate) resource_policy: Option<Policy>,
    pub(crate) kind: Kind,
}

impl Builder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn anonymous_reader(mut self, val: bool) -> Self {
        self.anonymous_reader = val;
        self
    }

    pub fn resource_policy(mut self, val: Policy) -> Self {
        self.resource_policy = Some(val);
        self
    }

    pub async fn build(&self) -> Result<Box<dyn Enforcer>, Error> {
        log::trace!("building a {}Enforcer", self.kind);
        Ok(match &self.kind {
            Kind::Policy => {
                let policy = self.resource_policy
                    .clone()
                    .ok_or(Error::PolicyRequired)?;
                self.build_with_policy(policy).await?
            }
            #[cfg(feature = "casbin")]
            Kind::Casbin(builder) => Box::new(
                CasbinEnforcer::new(
                    self.anonymous_reader,
                    &builder.base_policy,
                    &builder.default_model,
                    self.resource_policy.clone(),
                ).await?
            ),
        })
    }

    pub async fn build_with_policy(
        &self,
        mut policy: Policy,
    ) -> Result<Box<dyn Enforcer>, Error> {
        log::trace!("building a {}Enforcer with {policy:?}", self.kind);
        Ok(match &self.kind {
            Kind::Policy => {
                if self.anonymous_reader {
                    policy.agent_roles
                        .push((None, Role::Reader).into())
                }
                Box::new(PolicyEnforcer::from(policy))
            }
            #[cfg(feature = "casbin")]
            Kind::Casbin(builder) => Box::new(
                CasbinEnforcer::new(
                    self.anonymous_reader,
                    &builder.base_policy,
                    &builder.default_model,
                    Some(policy),
                ).await?,
            )
        })
    }
}

mod display {
    use std::fmt::{Display, Formatter, Result};
    use super::Kind;

    impl Display for Kind {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Kind::Policy => f.write_str("Policy"),
                #[cfg(feature = "casbin")]
                Kind::Casbin(..) => f.write_str("Casbin"),
            }
        }
    }
}
