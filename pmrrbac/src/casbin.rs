use casbin::{
    CoreApi,
    DefaultModel,
    MemoryAdapter,
    MgmtApi,
};
use pmrcore::ac::{
    agent::Agent,
    role::Role,
    genpolicy::{
        AgentRole,
        Policy,
        ResGrant,
        RolePermit,
    },
    traits::Enforcer,
};

use crate::{
    builder::{Kind, Builder},
    error::Error,
};

/// The casbin model for pmrapp.
const DEFAULT_MODEL: &str = "\
[request_definition]
r = sub, res, act

[policy_definition]
p = sub, res, act

[role_definition]
g = _, _, _
g2 = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = (g(r.sub, p.sub, r.res) || g(r.sub, p.sub, p.res) || g2(r.sub, p.sub)) && keyMatch2(r.res, p.res) && keyMatch(r.act, p.act)
";

/// The default policy for PMR
///
/// The fields are comma separated in the form of: role, route, action
///
/// role - the name of the role - it will be granted to agents.
/// route - the route to a given resource
/// action - the name for a given action
const DEFAULT_POLICIES: &str = "\
# Managers can do everything
manager, /*, *

# readers have limited access; this should be granted per resource
# reader, /*,
# reader, /*, protocol_read
# reader, /*, protocol_edit

# owners have everything, including granted access
owner, /*,                 # empty group signifies typical actions (e.g. the default view)
owner, /*, editor_view     # *_view allows the view of the editing UI (e.g. exposure wizard)
owner, /*, editor_edit     # *_edit allows the submission of edits, may be removed, see below
owner, /*, grant_view      # grant signifies being able to grant additional access
owner, /*, grant_edit
owner, /*, protocol_view   # protocol signifies git clone/etc
owner, /*, protocol_edit

# editors have everything, can see grants but cannot grant additional access to others
editor, /*,                 # empty group signifies typical actions (e.g. view)
editor, /*, editor_view
editor, /*, editor_edit
editor, /*, grant_view
editor, /*, protocol_view
editor, /*, protocol_edit
";

/// An alternative policy
///
/// This one prevents article owners from being able to edit content after publication
const ALT_POLICIES: &str = "\
# managers can do everything
manager, /*, *

# owners have everything, including granted access
owner, /*,                 # empty group signifies typical actions (e.g. view)
owner, /*, editor_view     # being able to view the editor does not mean changes be accepted by default,
# owner, /*, editor_edit   # if the associated *_edit action is removed.
owner, /*, grant_view      # grant signifies being able to grant additional access
owner, /*, grant_edit      # note that the implementation may need to figure out who's granting what
owner, /*, protocol_view   # protocol signifies git clone/etc
owner, /*, protocol_edit

# editors have everything, can see grants but cannot grant additional access to others
editor, /*, ,               # empty group signifies typical actions (e.g. view)
editor, /*, editor_view
editor, /*, editor_edit
editor, /*, grant_view
editor, /*, protocol_view
editor, /*, protocol_edit
";

/// Builds a role-based access controller (RBAC) for PMR.
///
/// Methods can be chained in order to set the configuration values.
/// The `CasbinEnforcer` is constructed by calling [`build`].
///
/// New instances of the builder can be obtained via `Builder::default`
/// or `Builder::new`.  The former provides nothing while the latter
/// provides the default policy.  A third version `Builder::new_limited`
/// uses the alternative base policy which prevents owners from editing
/// after publication.
#[derive(Clone, Debug, Default)]
pub struct CasbinBuilder {
    pub(crate) anonymous_reader: bool,
    pub(crate) base_policy: Box<str>,
    pub(crate) default_model: Box<str>,
    pub(crate) resource_policy: Option<Policy>,
}

impl From<CasbinBuilder> for Builder {
    fn from(mut builder: CasbinBuilder) -> Self {
        Self {
            anonymous_reader: builder.anonymous_reader,
            resource_policy: builder.resource_policy.take(),
            kind: Kind::Casbin(builder),
        }
    }
}

impl CasbinBuilder {
    pub fn new() -> Self {
        Self {
            base_policy: DEFAULT_POLICIES.into(),
            default_model: DEFAULT_MODEL.into(),
            .. Default::default()
        }
    }

    pub fn new_limited() -> Self {
        Self {
            base_policy: ALT_POLICIES.into(),
            default_model: DEFAULT_MODEL.into(),
            .. Default::default()
        }
    }

    pub fn anonymous_reader(mut self, val: bool) -> Self {
        self.anonymous_reader = val;
        self
    }

    pub fn base_policy(mut self, val: &str) -> Self {
        self.base_policy = val.into();
        self
    }

    pub fn default_model(mut self, val: &str) -> Self {
        self.default_model = val.into();
        self
    }

    pub fn resource_policy(mut self, val: Policy) -> Self {
        self.resource_policy = Some(val);
        self
    }

    pub async fn build(&self) -> Result<CasbinEnforcer, casbin::Error> {
        CasbinEnforcer::new(
            self.anonymous_reader,
            &self.base_policy,
            &self.default_model,
            self.resource_policy.clone(),
        ).await
    }

    pub async fn build_with_policy(
        &self,
        resource_policy: Policy,
    ) -> Result<CasbinEnforcer, casbin::Error> {
        log::trace!("{resource_policy:?}");
        CasbinEnforcer::new(
            self.anonymous_reader,
            &self.base_policy,
            &self.default_model,
            Some(resource_policy),
        ).await
    }
}

pub struct CasbinEnforcer {
    enforcer: casbin::Enforcer,
}

impl CasbinEnforcer {
    pub async fn new(
        anonymous_reader: bool,
        policies: &str,
        model: &str,
        resource_policy: Option<Policy>,
    ) -> Result<Self, casbin::Error> {
        let m = DefaultModel::from_str(model).await?;
        let a = MemoryAdapter::default();
        let mut enforcer = casbin::Enforcer::new(m, a).await?;
        let policies = policies.lines()
            .filter_map(|line| {
                let result = line
                    .split('#')
                    .next()
                    .expect("split must produce at least one token")
                    .split(", ")
                    .map(str::trim)
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                (result.len() == 3).then_some(result)
            })
            .collect::<Vec<_>>();
        let n = policies.len();
        enforcer.add_named_policies("p", policies).await?;
        log::debug!("new CasbinEnforcer set up with {n} policies");
        let mut result = Self { enforcer };
        if anonymous_reader {
            log::debug!("new CasbinEnforcer granting anonymous agent role reader");
            result.grant_agent_role(None::<&str>, Role::Reader).await?;
        }
        if let Some(resource_policy) = resource_policy {
            log::debug!("new CasbinEnforcer has additional resource policies");
            result.set_resource_policy(resource_policy).await?;
        }
        Ok(result)
    }

    fn to_agent(agent: Option<impl AsRef<str> + std::fmt::Display>) -> String {
        agent.map(|agent| format!("u:{agent}"))
            .unwrap_or("-".to_string())
    }

    /// Grant agent the role, which will enable the agent the role for
    /// resources that have a policy attached for the role.
    pub async fn grant_agent_role(
        &mut self,
        agent: Option<impl AsRef<str> + std::fmt::Display>,
        role: Role,
    ) -> Result<bool, casbin::Error> {
        self.enforcer.add_named_grouping_policy("g2", vec![
            Self::to_agent(agent),
            role.into(),
        ]).await
    }

    /// Revoke an implicit role from agent.
    pub async fn revoke_agent_role(
        &mut self,
        agent: impl AsRef<str> + std::fmt::Display,
        role: Role,
    ) -> Result<bool, casbin::Error> {
        self.enforcer.remove_named_grouping_policy("g2", vec![
            Self::to_agent(Some(agent)),
            role.into(),
        ]).await
    }

    /// Grant agent specified role at resource.
    /// Creates the relevant casbin grouping policy.
    pub async fn grant_res(
        &mut self,
        agent: Option<impl AsRef<str> + std::fmt::Display>,
        role: Role,
        resource: impl Into<String>,
    ) -> Result<bool, casbin::Error> {
        self.enforcer.add_named_grouping_policy("g", vec![
            Self::to_agent(agent),
            role.into(),
            resource.into(),
        ]).await
    }

    /// Revokes agent specified role at resource.
    /// Removes the relevant casbin grouping policy.
    pub async fn revoke_res(
        &mut self,
        agent: Option<impl AsRef<str> + std::fmt::Display>,
        role: Role,
        resource: impl Into<String>,
    ) -> Result<bool, casbin::Error> {
        self.enforcer.remove_named_grouping_policy("g", vec![
            Self::to_agent(agent),
            role.into(),
            resource.into(),
        ]).await
    }

    /// Attach a policy.
    pub async fn attach_policy(
        &mut self,
        role: Role,
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> Result<bool, casbin::Error> {
        self.enforcer.add_named_policy("p", vec![
            role.into(),
            resource.into(),
            action.into(),
        ]).await
    }

    /// Deattach a policy.
    pub async fn deattach_policy(
        &mut self,
        role: Role,
        resource: impl Into<String>,
        action: impl Into<String>,
    ) -> Result<bool, casbin::Error> {
        self.enforcer.remove_named_policy("p", vec![
            role.into(),
            resource.into(),
            action.into(),
        ]).await
    }

    pub async fn set_resource_policy(
        &mut self,
        policy: Policy,
    ) -> Result<(), casbin::Error> {
        for AgentRole { agent, role } in policy.agent_roles.into_iter() {
            self.grant_agent_role(agent, role).await?;
        }
        for ResGrant { res, agent, role } in policy.res_grants.into_iter() {
            self.grant_res(agent.as_ref(), role, res).await?;
        }
        for RolePermit { role, action } in policy.role_permits.into_iter() {
            self.attach_policy(role, policy.resource.clone(), action).await?;
        }
        Ok(())
    }

    /// Validates if the agent accessing the path has the required rights.
    fn casbin_enforce(
        &self,
        agent: Option<impl AsRef<str> + std::fmt::Display>,
        resource: impl AsRef<str>,
        action: impl AsRef<str>,
    ) -> Result<bool, casbin::Error> {
        self.enforcer.enforce((
            Self::to_agent(agent).as_str(),
            resource.as_ref(),
            action.as_ref(),
        ))
    }
}

impl Enforcer for CasbinEnforcer {
    type Error = Error;

    fn enforce(&self, agent: &Agent, res: &str, action: &str) -> Result<bool, Self::Error> {
        Ok(self.casbin_enforce(<Agent as Into<Option<String>>>::into(agent.clone()), res, action)?)
    }
}

#[cfg(test)]
mod test {
    use anyhow::{self, anyhow as err};
    use pmrcore::ac::user::User;
    use crate::builder::Builder;
    use super::*;

    fn mk_agent(name: &str) -> Agent {
        User { id: 0, name: name.to_string(), created_ts: 1 }.into()
    }

    #[tokio::test]
    async fn empty() -> anyhow::Result<()> {
        let mut security = CasbinBuilder::default()
            .default_model(DEFAULT_MODEL.into())
            .build()
            .await?;
        // the rules don't actually work without the default
        assert!(security.grant_res(Some("admin"), Role::Manager, "/*").await?);
        assert!(!security.enforce(&mk_agent("admin"), "/exposure/1", "")?);
        Ok(())
    }

    #[tokio::test]
    async fn demo() -> anyhow::Result<()> {
        let mut security = CasbinBuilder::new().build().await?;
        let not_logged_in: Option<&str> = None;

        // admin account has access to every part of the application
        assert!(security.grant_res(Some("admin"), Role::Manager, "/*").await?);
        // alice is the owner of exposure 1
        assert!(security.grant_agent_role(Some("alice"), Role::Reader).await?);
        assert!(security.grant_res(Some("alice"), Role::Owner, "/exposure/1").await?);
        // bob is the owner of exposure 2
        assert!(security.grant_agent_role(Some("bob"), Role::Reader).await?);
        assert!(security.grant_res(Some("bob"), Role::Owner, "/exposure/2").await?);
        // cathy is the editor of exposure 2
        assert!(security.grant_res(Some("cathy"), Role::Editor, "/exposure/2").await?);
        // create the anonymous agent also
        assert!(security.grant_agent_role(not_logged_in, Role::Reader).await?);
        // make site root public
        assert!(security.attach_policy(Role::Reader, "/", "").await?);
        // make /exposure/1 public
        assert!(security.attach_policy(Role::Reader, "/exposure/1", "").await?);
        // make /workspace/1 public, also clonable
        assert!(security.attach_policy(Role::Reader, "/workspace/1", "").await?);
        assert!(security.attach_policy(Role::Reader, "/workspace/1", "protocol_view").await?);

        // everybody should be able to read the site root and index page
        assert!(security.enforce(&Agent::Anonymous, "/", "")?);
        assert!(security.enforce(&mk_agent("alice"), "/", "")?);
        assert!(security.enforce(&mk_agent("bob"), "/", "")?);

        // alice being the owner, should be able to do everything in exposure 1
        assert!(security.enforce(&mk_agent("alice"), "/exposure/1", "")?);
        assert!(security.enforce(&mk_agent("alice"), "/exposure/1", "editor_view")?);
        assert!(security.enforce(&mk_agent("alice"), "/exposure/1", "editor_edit")?);
        assert!(security.enforce(&mk_agent("alice"), "/exposure/1", "grant_view")?);
        assert!(security.enforce(&mk_agent("alice"), "/exposure/1", "grant_edit")?);
        // but wouldn't be able to access management functions
        assert!(!security.enforce(&mk_agent("alice"), "/exposure/1", "manage_view")?);
        assert!(!security.enforce(&mk_agent("alice"), "/exposure/1", "manage_edit")?);
        // and shouldn't be able to read or write to the private exposure 2
        assert!(!security.enforce(&mk_agent("alice"), "/exposure/2", "")?);
        assert!(!security.enforce(&mk_agent("alice"), "/exposure/2", "editor_view")?);
        assert!(!security.enforce(&mk_agent("alice"), "/exposure/2", "editor_edit")?);
        assert!(!security.enforce(&mk_agent("alice"), "/exposure/2", "grant_view")?);
        assert!(!security.enforce(&mk_agent("alice"), "/exposure/2", "grant_edit")?);

        // bob can read exposure 1, but cannot edit
        assert!(security.enforce(&mk_agent("bob"), "/exposure/1", "")?);
        assert!(!security.enforce(&mk_agent("bob"), "/exposure/1", "editor_view")?);
        assert!(!security.enforce(&mk_agent("bob"), "/exposure/1", "editor_edit")?);
        // bob is the natural owner of exposure 2 so can do everything
        assert!(security.enforce(&mk_agent("bob"), "/exposure/2", "")?);
        assert!(security.enforce(&mk_agent("bob"), "/exposure/2", "editor_view")?);
        assert!(security.enforce(&mk_agent("bob"), "/exposure/2", "editor_edit")?);
        assert!(security.enforce(&mk_agent("bob"), "/exposure/2", "grant_view")?);
        assert!(security.enforce(&mk_agent("bob"), "/exposure/2", "grant_edit")?);

        // cathy is only an editor so she wouldn't be able to grant additional access for others
        assert!(security.enforce(&mk_agent("cathy"), "/exposure/2", "")?);
        assert!(security.enforce(&mk_agent("cathy"), "/exposure/2", "editor_view")?);
        assert!(security.enforce(&mk_agent("cathy"), "/exposure/2", "editor_edit")?);
        assert!(security.enforce(&mk_agent("cathy"), "/exposure/2", "grant_view")?);
        assert!(!security.enforce(&mk_agent("cathy"), "/exposure/2", "grant_edit")?);

        // not logged in agents can only view exposure 1
        assert!(security.enforce(&Agent::Anonymous, "/exposure/1", "")?);
        assert!(!security.enforce(&Agent::Anonymous, "/exposure/2", "")?);

        // not logged in agents cannot access any of the editor functions
        assert!(!security.enforce(&Agent::Anonymous, "/exposure/1", "editor_view")?);
        assert!(!security.enforce(&Agent::Anonymous, "/exposure/1", "editor_edit")?);
        assert!(!security.enforce(&Agent::Anonymous, "/exposure/2", "editor_view")?);
        assert!(!security.enforce(&Agent::Anonymous, "/exposure/2", "editor_edit")?);

        // the admin can do everything so far
        assert!(security.enforce(&mk_agent("admin"), "/exposure/1", "")?);
        assert!(security.enforce(&mk_agent("admin"), "/exposure/1", "editor_view")?);
        assert!(security.enforce(&mk_agent("admin"), "/exposure/1", "editor_edit")?);
        assert!(security.enforce(&mk_agent("admin"), "/exposure/1", "grant_view")?);
        assert!(security.enforce(&mk_agent("admin"), "/exposure/1", "grant_edit")?);
        assert!(security.enforce(&mk_agent("admin"), "/exposure/1", "manage_view")?);
        assert!(security.enforce(&mk_agent("admin"), "/exposure/1", "manage_edit")?);
        assert!(security.enforce(&mk_agent("admin"), "/exposure/2", "")?);
        assert!(security.enforce(&mk_agent("admin"), "/exposure/2", "editor_view")?);
        assert!(security.enforce(&mk_agent("admin"), "/exposure/2", "editor_edit")?);
        assert!(security.enforce(&mk_agent("admin"), "/exposure/2", "grant_view")?);
        assert!(security.enforce(&mk_agent("admin"), "/exposure/2", "grant_edit")?);

        Ok(())
    }

    #[tokio::test]
    async fn policy_usage_private() -> anyhow::Result<()> {
        let mut security = CasbinBuilder::new().build().await?;
        // Casbin's policy doesn't verify the agent field of Policy.
        security.set_resource_policy(serde_json::from_str(r#"{
            "agent": "Anonymous",
            "resource": "/item/1",
            "agent_roles": [],
            "res_grants": [
                {"res": "/*", "agent": "admin", "role": "Manager"},
                {"res": "/item/1", "agent": "alice", "role": "Owner"}
            ],
            "role_permits": [
                {"role": "Owner", "action": "editor_view"},
                {"role": "Owner", "action": "editor_edit"}
            ]
        }"#)?).await?;

        assert!(security.enforce(&mk_agent("admin"), "/item/1", "")?);
        assert!(security.enforce(&mk_agent("admin"), "/item/1", "editor_view")?);
        assert!(security.enforce(&mk_agent("admin"), "/item/1", "editor_edit")?);
        assert!(security.enforce(&mk_agent("alice"), "/item/1", "")?);
        assert!(security.enforce(&mk_agent("alice"), "/item/1", "editor_view")?);
        assert!(security.enforce(&mk_agent("alice"), "/item/1", "editor_edit")?);
        assert!(!security.enforce(&Agent::Anonymous, "/item/1", "")?);
        assert!(!security.enforce(&Agent::Anonymous, "/item/1", "editor_view")?);
        assert!(!security.enforce(&Agent::Anonymous, "/item/1", "editor_edit")?);

        Ok(())
    }

    #[tokio::test]
    async fn policy_usage_reviewer() -> anyhow::Result<()> {
        // a rough approximation of a resource under review, with reviewer
        // having **global** access to everything
        let policy: Policy = serde_json::from_str(r#"{
            "agent": "Anonymous",
            "resource": "/item/1",
            "agent_roles": [
                {"agent": "reviewer", "role": "Reader"},
                {"agent": "reviewer", "role": "Reviewer"}
            ],
            "res_grants": [
                {"res": "/*", "agent": "reviewer", "role": "Reviewer"},
                {"res": "/item/1", "agent": "alice", "role": "Owner"}
            ],
            "role_permits": [
                {"role": "Reviewer", "action": ""},
                {"role": "Reviewer", "action": "editor_view"},
                {"role": "Reviewer", "action": "editor_edit"}
            ]
        }"#)?;

        let security = CasbinBuilder::new()
            .anonymous_reader(true)
            .resource_policy(policy.clone())
            .build()
            .await?;
        assert!(!security.enforce(&Agent::Anonymous, "/item/1", "")?);
        assert!(security.enforce(&mk_agent("alice"), "/item/1", "")?);
        // this doesn't have the restriction the limited version poses
        assert!(security.enforce(&mk_agent("alice"), "/item/1", "editor_view")?);
        assert!(security.enforce(&mk_agent("alice"), "/item/1", "editor_edit")?);
        assert!(security.enforce(&mk_agent("reviewer"), "/item/1", "")?);
        assert!(security.enforce(&mk_agent("reviewer"), "/item/1", "editor_view")?);
        assert!(security.enforce(&mk_agent("reviewer"), "/item/1", "editor_edit")?);
        // this wasn't granted globally by the model, so this should have no effect.
        assert!(!security.enforce(&mk_agent("reviewer"), "/item/1", "grant_edit")?);

        assert!(!security.enforce(&mk_agent("reviewer"), "/item/2", "")?);
        assert!(!security.enforce(&mk_agent("reviewer"), "/item/2", "editor_view")?);
        assert!(!security.enforce(&mk_agent("reviewer"), "/item/2", "editor_edit")?);

        let security = CasbinBuilder::new_limited()
            .anonymous_reader(true)
            .resource_policy(policy.clone())
            .build()
            .await?;
        assert!(!security.enforce(&Agent::Anonymous, "/item/1", "")?);
        assert!(security.enforce(&mk_agent("alice"), "/item/1", "")?);
        // the limited model restricts owners from submitting edits for
        // resources under review, but has no restrictions on viewing the
        // edit form.
        assert!(security.enforce(&mk_agent("alice"), "/item/1", "editor_view")?);
        assert!(!security.enforce(&mk_agent("alice"), "/item/1", "editor_edit")?);
        assert!(security.enforce(&mk_agent("reviewer"), "/item/1", "")?);
        assert!(security.enforce(&mk_agent("reviewer"), "/item/1", "editor_view")?);
        assert!(security.enforce(&mk_agent("reviewer"), "/item/1", "editor_edit")?);
        // this wasn't granted globally by the model, so this should have no effect.
        assert!(!security.enforce(&mk_agent("reviewer"), "/item/1", "grant_edit")?);

        assert!(!security.enforce(&mk_agent("reviewer"), "/item/2", "")?);
        assert!(!security.enforce(&mk_agent("reviewer"), "/item/2", "editor_view")?);
        assert!(!security.enforce(&mk_agent("reviewer"), "/item/2", "editor_edit")?);

        Ok(())
    }

    #[tokio::test]
    async fn policy_usage_reviewer_unconstrained() -> anyhow::Result<()> {
        // a rough approximation of a resource under review
        let policy: Policy = serde_json::from_str(r#"{
            "agent": "Anonymous",
            "resource": "/item/1",
            "agent_roles": [],
            "res_grants": [
                {"res": "/*", "agent": "reviewer", "role": "Reviewer"},
                {"res": "/item/1", "agent": "alice", "role": "Owner"}
            ],
            "role_permits": [
                {"role": "Reviewer", "action": ""},
                {"role": "Reviewer", "action": "editor_view"},
                {"role": "Reviewer", "action": "editor_edit"}
            ]
        }"#)?;


        let mut security = CasbinBuilder::new()
            .anonymous_reader(true)
            .resource_policy(policy.clone())
            .build()
            .await?;
        // the resource policy doesn't provide additional underlying policies, so this is
        // manually unconstrained to emulate the default policies defined for editor/manager
        security.attach_policy(Role::Reviewer, "/*", "").await?;
        security.attach_policy(Role::Reviewer, "/*", "editor_view").await?;
        security.attach_policy(Role::Reviewer, "/*", "editor_edit").await?;

        assert!(security.enforce(&mk_agent("reviewer"), "/item/1", "")?);
        assert!(security.enforce(&mk_agent("reviewer"), "/item/1", "editor_view")?);
        assert!(security.enforce(&mk_agent("reviewer"), "/item/1", "editor_edit")?);
        // note that /item/2 will also be permitted despite the resource only has /item/1
        assert!(security.enforce(&mk_agent("reviewer"), "/item/2", "")?);
        assert!(security.enforce(&mk_agent("reviewer"), "/item/2", "editor_view")?);
        assert!(security.enforce(&mk_agent("reviewer"), "/item/2", "editor_edit")?);
        // anyway, the model may be possible to have further simplification.
        Ok(())
    }

    #[tokio::test]
    async fn policy_usage_published() -> anyhow::Result<()> {
        let security = CasbinBuilder::new()
            .anonymous_reader(true)
            .resource_policy(serde_json::from_str(r#"{
                "agent": "Anonymous",
                "resource": "/item/1",
                "agent_roles": [],
                "res_grants": [
                    {"res": "/*", "agent": "admin", "role": "Manager"},
                    {"res": "/item/1", "agent": "alice", "role": "Owner"}
                ],
                "role_permits": [
                    {"role": "Owner", "action": "editor_view"},
                    {"role": "Reader", "action": ""}
                ]
            }"#)?)
            .build()
            .await?;
        // a rough approximation of a published resource

        assert!(security.enforce(&mk_agent("admin"), "/item/1", "")?);
        assert!(security.enforce(&mk_agent("admin"), "/item/1", "editor_view")?);
        assert!(security.enforce(&mk_agent("admin"), "/item/1", "editor_edit")?);
        assert!(security.enforce(&mk_agent("alice"), "/item/1", "")?);
        assert!(security.enforce(&mk_agent("alice"), "/item/1", "editor_view")?);
        assert!(security.enforce(&mk_agent("alice"), "/item/1", "editor_edit")?);
        assert!(security.enforce(&Agent::Anonymous, "/item/1", "")?);
        assert!(!security.enforce(&Agent::Anonymous, "/item/1", "editor_view")?);
        assert!(!security.enforce(&Agent::Anonymous, "/item/1", "editor_edit")?);

        Ok(())
    }

    #[tokio::test]
    async fn policy_usage_published_alt() -> anyhow::Result<()> {
        let security = CasbinBuilder::new_limited()
            .anonymous_reader(true)
            .resource_policy(serde_json::from_str(r#"{
                "agent": "Anonymous",
                "resource": "/item/1",
                "agent_roles": [],
                "res_grants": [
                    {"res": "/*", "agent": "admin", "role": "Manager"},
                    {"res": "/item/1", "agent": "alice", "role": "Owner"}
                ],
                "role_permits": [
                    {"role": "Owner", "action": "editor_view"},
                    {"role": "Reader", "action": ""}
                ]
            }"#)?)
            .build()
            .await?;
        // a rough approximation of a published resource

        assert!(security.enforce(&mk_agent("admin"), "/item/1", "")?);
        assert!(security.enforce(&mk_agent("admin"), "/item/1", "editor_view")?);
        assert!(security.enforce(&mk_agent("admin"), "/item/1", "editor_edit")?);
        assert!(security.enforce(&mk_agent("alice"), "/item/1", "")?);
        assert!(security.enforce(&mk_agent("alice"), "/item/1", "editor_view")?);
        assert!(!security.enforce(&mk_agent("alice"), "/item/1", "editor_edit")?);
        assert!(security.enforce(&Agent::Anonymous, "/item/1", "")?);
        assert!(!security.enforce(&Agent::Anonymous, "/item/1", "editor_view")?);
        assert!(!security.enforce(&Agent::Anonymous, "/item/1", "editor_edit")?);

        Ok(())
    }

    struct EnforcerTester {
        casbin: Box<dyn crate::Enforcer>,  // CasbinEnforcer,
        pe: Box<dyn crate::Enforcer>,  // PolicyEnforcer,
    }

    impl EnforcerTester {
        async fn new(builder: CasbinBuilder, policy: Policy) -> anyhow::Result<Self> {
            let casbin = Builder::from(builder)
                .resource_policy(policy.clone())
                .anonymous_reader(true)
                .build()
                .await?;
            let pe = Builder::new()
                .resource_policy(policy)
                .anonymous_reader(true)
                .build()
                .await?;
            Ok(Self { pe, casbin })
        }

        // PolicyEnforcer at this stage will only validate a subset of the
        // policy by design, as it requires explicit grants (in the main
        // system they are associated directly with the workflow state of
        // the particular resource), and so validation here may still deny
        // access that the Casbin based enforcer will allow (simply because
        // it process the wild cards and the graph).
        fn check_granted_casbin(
            &self,
            agent: &Agent,
            resource: &str,
            action: &str,
        ) -> anyhow::Result<()> {
            self.casbin.enforce(agent, resource, action)?
                .then_some(())
                .ok_or(err!("CasbinEnforcer denied when expecting granted"))
        }

        fn check_granted_policy(
            &self,
            agent: &Agent,
            resource: &str,
            action: &str,
        ) -> anyhow::Result<()> {
            self.pe.enforce(agent, resource, action)?
                .then_some(())
                .ok_or(err!("PolicyEnforcer denied when expecting granted"))
        }

        // The version where both PolicyEnforcer and Casbin version will
        // both grant access because the information are complete for both.
        fn check_granted(
            &self,
            agent: &Agent,
            resource: &str,
            action: &str,
        ) -> anyhow::Result<()> {
            self.casbin.enforce(agent, resource, action)?
                .then_some(())
                .ok_or(err!("CasbinEnforcer denied when expecting granted"))?;
            self.pe.enforce(agent, resource, action)?
                .then_some(())
                .ok_or(err!("PolicyEnforcer denied when expecting granted"))
        }

        // PolicyEnforcer should NOT be permitting access when the Casbin
        // based implementation have denied it, as the basic version only
        // should provide a subset validation (i.e without the graph).
        fn check_denied(
            &self,
            agent: &Agent,
            resource: &str,
            action: &str,
        ) -> anyhow::Result<()> {
            (!self.casbin.enforce(agent, resource, action)?)
                .then_some(())
                .ok_or(err!("CasbinEnforcer granted when expecting denied"))?;
            (!self.pe.enforce(agent, resource, action)?)
                .then_some(())
                .ok_or(err!("PolicyEnforcer granted when expecting denied"))
        }
    }

    #[tokio::test]
    async fn comparison_policy_usage_private() -> anyhow::Result<()> {
        let tester = EnforcerTester::new(
            CasbinBuilder::new()
                .anonymous_reader(true),
            serde_json::from_str(r#"{
                "agent": {
                    "User": {
                        "id": 0,
                        "name": "alice",
                        "created_ts": 1
                    }
                },
                "resource": "/item/1",
                "agent_roles": [],
                "res_grants": [
                    {"res": "/item/1", "agent": "alice", "role": "Owner"}
                ],
                "role_permits": [
                    {"role": "Owner", "action": "editor_view"},
                    {"role": "Owner", "action": "editor_edit"}
                ]
            }"#)?
        ).await?;

        assert!(tester.check_granted_casbin(&mk_agent("alice"), "/item/1", "").is_ok());
        assert!(tester.check_granted(&mk_agent("alice"), "/item/1", "").is_err());
        assert!(tester.check_granted(&mk_agent("alice"), "/item/1", "editor_view").is_ok());
        assert!(tester.check_granted(&mk_agent("alice"), "/item/1", "editor_edit").is_ok());
        assert!(tester.check_denied(&mk_agent("alice"), "/item/1", "grants").is_ok());
        assert!(tester.check_denied(&mk_agent("alice"), "/item/1", "manage").is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn comparison_policy_usage_manager() -> anyhow::Result<()> {
        let tester = EnforcerTester::new(
            CasbinBuilder::new()
                .anonymous_reader(true),
            serde_json::from_str(r#"{
                "agent": {
                    "User": {
                        "id": 0,
                        "name": "admin",
                        "created_ts": 1
                    }
                },
                "resource": "/item/1",
                "agent_roles": [{
                    "agent": "admin",
                    "role": "Manager"
                }],
                "res_grants": [
                ],
                "role_permits": [{
                    "role": "Manager",
                    "action": "*"
                }]
            }"#)?
        ).await?;

        // with an additional wildcard action, both enforcers will work
        assert!(tester.check_granted(&mk_agent("admin"), "/item/1", "").is_ok());
        assert!(tester.check_granted(&mk_agent("admin"), "/item/1", "custom1").is_ok());
        assert!(tester.check_granted(&mk_agent("admin"), "/item/1", "custom2").is_ok());
        assert!(tester.check_granted(&mk_agent("admin"), "/item/1", "manage").is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn comparison_policy_usage_manager_wildcard_conflict() -> anyhow::Result<()> {
        let tester = EnforcerTester::new(
            CasbinBuilder::new()
                .anonymous_reader(true),
            serde_json::from_str(r#"{
                "agent": {
                    "User": {
                        "id": 0,
                        "name": "admin",
                        "created_ts": 1
                    }
                },
                "resource": "/item/1",
                "agent_roles": [{
                    "agent": "admin",
                    "role": "Manager"
                }],
                "res_grants": [],
                "role_permits": [
                    {"role": "Owner", "action": ""},
                    {"role": "Manager", "action": "*"}
                ]
            }"#)?
        ).await?;

        // validate that provided role will not conflict wildcard
        assert!(tester.check_granted(&mk_agent("admin"), "/item/1", "").is_ok());
        assert!(tester.check_granted(&mk_agent("admin"), "/item/1", "custom1").is_ok());
        assert!(tester.check_granted(&mk_agent("admin"), "/item/1", "custom2").is_ok());
        assert!(tester.check_granted(&mk_agent("admin"), "/item/1", "manage").is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn comparison_policy_usage_casbin_default() -> anyhow::Result<()> {
        let tester = EnforcerTester::new(
            CasbinBuilder::default(),
            serde_json::from_str(r#"{
                "agent": {
                    "User": {
                        "id": 0,
                        "name": "admin",
                        "created_ts": 1
                    }
                },
                "resource": "/item/1",
                "agent_roles": [{
                    "agent": "admin",
                    "role": "Manager"
                }],
                "res_grants": [
                ],
                "role_permits": [
                    {"role": "Owner", "action": ""},
                    {"role": "Manager", "action": "grant"}
                ]
            }"#)?
        ).await?;

        // default action was only granted to owner, which manager is not
        assert!(tester.check_granted_casbin(&mk_agent("admin"), "/item/1", "").is_err());
        // due to the way graphs work, nothing actually works without default wildcard grants
        assert!(tester.check_granted_casbin(&mk_agent("admin"), "/item/1", "grant").is_err());

        // but the PolicyEnforcer will just work as it doesn't do fancy graph traversal
        assert!(tester.check_granted_policy(&mk_agent("admin"), "/item/1", "grant").is_ok());

        // no manager permitted
        assert!(tester.check_denied(&mk_agent("admin"), "/item/1", "manage").is_err());

        Ok(())
    }
}
