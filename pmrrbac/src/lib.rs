use casbin::{
    CoreApi,
    DefaultModel,
    Enforcer,
    MemoryAdapter,
    MgmtApi,
};
use pmrcore::ac::{
    role::Role,
    genpolicy::{
        Policy,
        ResGrant,
        RolePermit,
        UserRole,
    },
};

pub mod error {
    pub use casbin::Error;
}
use crate::error::Error;

/// The casbin model for pmrapp.
const DEFAULT_MODEL: &str = "\
[request_definition]
r = sub, res, ep, met

[policy_definition]
p = sub, res, ep, met

[role_definition]
g = _, _, _
g2 = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = (g(r.sub, p.sub, r.res) || g(r.sub, p.sub, p.res) || g2(r.sub, p.sub)) && keyMatch2(r.res, p.res) && keyMatch(r.ep, p.ep) && r.met == p.met
";

// this matcher _doesn't_ work for the Rust version, so we just do another check for the None agent even if there is Some(...) agent.
// m = (g(r.sub, p.sub, r.res) || g(r.sub, p.sub, p.res) || g("-", p.sub, r.res) || g("-", p.sub, p.res)) && keyMatch2(r.res, p.res) && keyMatch(r.ep, p.ep) && r.met == p.met

/// The default policy for PMR
///
/// The fields are comma separated in the form of: role, route, endpoint group, HTTP method
///
/// role - the name of the role - it will be granted to agents.
/// route - the route to a given resource
/// endpoint group - the name for a given group of endpoints that share something in common
/// HTTP method - the permitted HTTP method associated with the policy
const DEFAULT_POLICIES: &str = "\
# Managers can do everything
Manager, /*, *, GET
Manager, /*, *, POST

# Readers have limited access; this should be granted per resource
# Reader, /*, , GET
# Reader, /*, protocol, GET

# Owners have everything, including granted access
Owner, /*, , GET           # empty group signifies typical actions (e.g. view)
Owner, /*, edit, GET       # edit signifies being able to edit content (e.g. exposure wizard)
Owner, /*, edit, POST      # this may be removed to prevent published content being edited
Owner, /*, grant, GET      # grant signifies being able to grant additional access
Owner, /*, grant, POST
Owner, /*, protocol, GET   # protocol signifies git clone/etc
Owner, /*, protocol, POST

# Editors have everything, can see grants but cannot grant additional access to others
Editor, /*, , GET           # empty group signifies typical actions (e.g. view)
Editor, /*, edit, GET
Editor, /*, edit, POST
Editor, /*, grant, GET
Editor, /*, protocol, GET
Editor, /*, protocol, POST
";

/// An alternative policy
///
/// This one prevents article owners from being able to edit content after publication
const ALT_POLICIES: &str = "\
# Managers can do everything
Manager, /*, *, GET
Manager, /*, *, POST

# Owners have everything, including granted access
Owner, /*, , GET           # empty group signifies typical actions (e.g. view)
Owner, /*, edit, GET       # edit signifies being able to edit content (e.g. exposure wizard)
# Owner, /*, edit, POST    # removing this prevents owners from being able to edit by default
Owner, /*, grant, GET      # grant signifies being able to grant additional access
Owner, /*, grant, POST     # note that the implementation may need to figure out who's granting what
Owner, /*, protocol, GET   # protocol signifies git clone/etc
Owner, /*, protocol, POST

# Editors have everything, can see grants but cannot grant additional access to others
Editor, /*, , GET           # empty group signifies typical actions (e.g. view)
Editor, /*, edit, GET
Editor, /*, edit, POST
Editor, /*, grant, GET
Editor, /*, protocol, GET
Editor, /*, protocol, POST
";

/// Builds a role-based access controller (RBAC) for PMR.
///
/// Methods can be chained in order to set the configuration values.
/// The `PmrRbac` is constructed by calling [`build`].
///
/// New instances of the builder can be obtained via `Builder::default`
/// or `Builder::new`.  The former provides nothing while the latter
/// provides the default policy.  A third version `Builder::new_limited`
/// uses the alternative base policy which prevents owners from editing
/// after publication.
#[derive(Clone, Debug, Default)]
pub struct Builder {
    anonymous_reader: bool,
    base_policy: Box<str>,
    default_model: Box<str>,
    resource_policy: Option<Policy>,
}

impl Builder {
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

    pub async fn build(&self) -> Result<PmrRbac, Error> {
        PmrRbac::new(
            self.anonymous_reader,
            &self.base_policy,
            &self.default_model,
            self.resource_policy.clone(),
        ).await
    }

    pub async fn build_with_resource_policy(
        &self,
        resource_policy: Policy,
    ) -> Result<PmrRbac, Error> {
        PmrRbac::new(
            self.anonymous_reader,
            &self.base_policy,
            &self.default_model,
            Some(resource_policy),
        ).await
    }
}

pub struct PmrRbac {
    enforcer: Enforcer,
}

impl PmrRbac {
    pub async fn new(
        anonymous_reader: bool,
        policies: &str,
        model: &str,
        resource_policy: Option<Policy>,
    ) -> Result<Self, Error> {
        let m = DefaultModel::from_str(model).await?;
        let a = MemoryAdapter::default();
        let mut enforcer = Enforcer::new(m, a).await?;
        let policies = policies.lines()
            .filter_map(|line| {
                let result = line
                    .split('#')
                    .next()
                    .expect("split must produce at least one token")
                    .trim()
                    .split(", ")
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                (result.len() == 4).then_some(result)
            })
            .collect::<Vec<_>>();
        let n = policies.len();
        enforcer.add_named_policies("p", policies).await?;
        log::info!("new enforcer set up with {n} policies");
        let mut result = Self { enforcer };
        if anonymous_reader {
            result.grant_agent_role(None::<&str>, Role::Reader).await?;
        }
        if let Some(resource_policy) = resource_policy {
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
    ) -> Result<bool, Error> {
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
    ) -> Result<bool, Error> {
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
    ) -> Result<bool, Error> {
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
    ) -> Result<bool, Error> {
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
        endpoint_group: impl Into<String>,
        http_method: impl Into<String>,
    ) -> Result<bool, Error> {
        self.enforcer.add_named_policy("p", vec![
            role.into(),
            resource.into(),
            endpoint_group.into(),
            http_method.into(),
        ]).await
    }

    /// Deattach a policy.
    pub async fn deattach_policy(
        &mut self,
        role: Role,
        resource: impl Into<String>,
        endpoint_group: impl Into<String>,
        http_method: impl Into<String>,
    ) -> Result<bool, Error> {
        self.enforcer.remove_named_policy("p", vec![
            role.into(),
            resource.into(),
            endpoint_group.into(),
            http_method.into(),
        ]).await
    }

    pub async fn set_resource_policy(
        &mut self,
        policy: Policy,
    ) -> Result<(), Error> {
        for UserRole { user, role } in policy.user_roles.into_iter() {
            self.grant_agent_role(Some(user), role).await?;
        }
        for ResGrant { res, agent, role } in policy.res_grants.into_iter() {
            self.grant_res(agent.as_ref(), role, res).await?;
        }
        for RolePermit { role, endpoint_group, method } in policy.role_permits.into_iter() {
            self.attach_policy(role, policy.resource.clone(), endpoint_group, method).await?;
        }
        Ok(())
    }

    /// Validates if the agent accessing the path has the required rights.
    pub fn enforce(
        &self,
        agent: Option<impl AsRef<str> + std::fmt::Display>,
        resource: impl AsRef<str>,
        endpoint_group: impl AsRef<str>,
        http_method: impl AsRef<str>,
    ) -> Result<bool, Error> {
        self.enforcer.enforce((
            Self::to_agent(agent).as_str(),
            resource.as_ref(),
            endpoint_group.as_ref(),
            http_method.as_ref(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn empty() -> anyhow::Result<()> {
        let mut security = Builder::default()
            .default_model(DEFAULT_MODEL.into())
            .build()
            .await?;
        // the rules don't actually work without the default
        assert!(security.grant_res(Some("admin"), Role::Manager, "/*").await?);
        assert!(!security.enforce(Some("admin"), "/exposure/1", "", "GET")?);
        Ok(())
    }

    #[tokio::test]
    async fn demo() -> anyhow::Result<()> {
        let mut security = Builder::new().build().await?;
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
        assert!(security.attach_policy(Role::Reader, "/", "", "GET").await?);
        // make /exposure/1 public
        assert!(security.attach_policy(Role::Reader, "/exposure/1", "", "GET").await?);
        // make /workspace/1 public, also clonable
        assert!(security.attach_policy(Role::Reader, "/workspace/1", "", "GET").await?);
        assert!(security.attach_policy(Role::Reader, "/workspace/1", "protocol", "GET").await?);

        // everybody should be able to read the site root and index page
        assert!(security.enforce(not_logged_in, "/", "", "GET")?);
        assert!(security.enforce(Some("alice"), "/", "", "GET")?);
        assert!(security.enforce(Some("bob"), "/", "", "GET")?);

        // alice being the owner, should be able to do everything in exposure 1
        assert!(security.enforce(Some("alice"), "/exposure/1", "", "GET")?);
        assert!(security.enforce(Some("alice"), "/exposure/1", "edit", "GET")?);
        assert!(security.enforce(Some("alice"), "/exposure/1", "edit", "POST")?);
        assert!(security.enforce(Some("alice"), "/exposure/1", "grant", "GET")?);
        assert!(security.enforce(Some("alice"), "/exposure/1", "grant", "POST")?);
        // but wouldn't be able to access management functions
        assert!(!security.enforce(Some("alice"), "/exposure/1", "manage", "GET")?);
        assert!(!security.enforce(Some("alice"), "/exposure/1", "manage", "POST")?);
        // and shouldn't be able to read or write to the private exposure 2
        assert!(!security.enforce(Some("alice"), "/exposure/2", "", "GET")?);
        assert!(!security.enforce(Some("alice"), "/exposure/2", "edit", "GET")?);
        assert!(!security.enforce(Some("alice"), "/exposure/2", "edit", "POST")?);
        assert!(!security.enforce(Some("alice"), "/exposure/2", "grant", "GET")?);
        assert!(!security.enforce(Some("alice"), "/exposure/2", "grant", "POST")?);

        // bob can read exposure 1, but cannot edit
        assert!(security.enforce(Some("bob"), "/exposure/1", "", "GET")?);
        assert!(!security.enforce(Some("bob"), "/exposure/1", "edit", "GET")?);
        assert!(!security.enforce(Some("bob"), "/exposure/1", "edit", "POST")?);
        // bob is the natural owner of exposure 2 so can do everything
        assert!(security.enforce(Some("bob"), "/exposure/2", "", "GET")?);
        assert!(security.enforce(Some("bob"), "/exposure/2", "edit", "GET")?);
        assert!(security.enforce(Some("bob"), "/exposure/2", "edit", "POST")?);
        assert!(security.enforce(Some("bob"), "/exposure/2", "grant", "GET")?);
        assert!(security.enforce(Some("bob"), "/exposure/2", "grant", "POST")?);

        // cathy is only an editor so she wouldn't be able to grant additional access for others
        assert!(security.enforce(Some("cathy"), "/exposure/2", "", "GET")?);
        assert!(security.enforce(Some("cathy"), "/exposure/2", "edit", "GET")?);
        assert!(security.enforce(Some("cathy"), "/exposure/2", "edit", "POST")?);
        assert!(security.enforce(Some("cathy"), "/exposure/2", "grant", "GET")?);
        assert!(!security.enforce(Some("cathy"), "/exposure/2", "grant", "POST")?);

        // not logged in agents can only view exposure 1
        assert!(security.enforce(not_logged_in, "/exposure/1", "", "GET")?);
        assert!(!security.enforce(not_logged_in, "/exposure/2", "", "GET")?);

        // not logged in agents cannot edit/POST etc
        assert!(!security.enforce(not_logged_in, "/exposure/1", "edit", "GET")?);
        assert!(!security.enforce(not_logged_in, "/exposure/1", "edit", "POST")?);
        assert!(!security.enforce(not_logged_in, "/exposure/2", "edit", "GET")?);
        assert!(!security.enforce(not_logged_in, "/exposure/2", "edit", "POST")?);

        // the admin can do everything so far
        assert!(security.enforce(Some("admin"), "/exposure/1", "", "GET")?);
        assert!(security.enforce(Some("admin"), "/exposure/1", "edit", "GET")?);
        assert!(security.enforce(Some("admin"), "/exposure/1", "edit", "POST")?);
        assert!(security.enforce(Some("admin"), "/exposure/1", "grant", "GET")?);
        assert!(security.enforce(Some("admin"), "/exposure/1", "grant", "POST")?);
        assert!(security.enforce(Some("admin"), "/exposure/1", "manage", "GET")?);
        assert!(security.enforce(Some("admin"), "/exposure/1", "manage", "POST")?);
        assert!(security.enforce(Some("admin"), "/exposure/2", "", "GET")?);
        assert!(security.enforce(Some("admin"), "/exposure/2", "edit", "GET")?);
        assert!(security.enforce(Some("admin"), "/exposure/2", "edit", "POST")?);
        assert!(security.enforce(Some("admin"), "/exposure/2", "grant", "GET")?);
        assert!(security.enforce(Some("admin"), "/exposure/2", "grant", "POST")?);

        Ok(())
    }

    #[tokio::test]
    async fn policy_usage_private() -> anyhow::Result<()> {
        let not_logged_in: Option<&str> = None;
        let mut security = Builder::new().build().await?;
        // a rough approximation of a private resource
        security.set_resource_policy(serde_json::from_str(r#"{
            "resource": "/item/1",
            "user_roles": [],
            "res_grants": [
                {"res": "/*", "agent": "admin", "role": "Manager"},
                {"res": "/item/1", "agent": "alice", "role": "Owner"}
            ],
            "role_permits": [
                {"role": "Owner", "endpoint_group": "edit", "method": "GET"},
                {"role": "Owner", "endpoint_group": "edit", "method": "POST"}
            ]
        }"#)?).await?;

        assert!(security.enforce(Some("admin"), "/item/1", "", "GET")?);
        assert!(security.enforce(Some("admin"), "/item/1", "edit", "GET")?);
        assert!(security.enforce(Some("admin"), "/item/1", "edit", "POST")?);
        assert!(security.enforce(Some("alice"), "/item/1", "", "GET")?);
        assert!(security.enforce(Some("alice"), "/item/1", "edit", "GET")?);
        assert!(security.enforce(Some("alice"), "/item/1", "edit", "POST")?);
        assert!(!security.enforce(not_logged_in, "/item/1", "", "GET")?);
        assert!(!security.enforce(not_logged_in, "/item/1", "edit", "GET")?);
        assert!(!security.enforce(not_logged_in, "/item/1", "edit", "POST")?);

        Ok(())
    }

    #[tokio::test]
    async fn policy_usage_reviewer() -> anyhow::Result<()> {
        let not_logged_in: Option<&str> = None;
        // a rough approximation of a resource under review, with reviewer
        // having **global** access to everything
        let policy: Policy = serde_json::from_str(r#"{
            "resource": "/item/1",
            "user_roles": [
                {"user": "reviewer", "role": "Reader"},
                {"user": "reviewer", "role": "Reviewer"}
            ],
            "res_grants": [
                {"res": "/*", "agent": "reviewer", "role": "Reviewer"},
                {"res": "/item/1", "agent": "alice", "role": "Owner"}
            ],
            "role_permits": [
                {"role": "Reviewer", "endpoint_group": "", "method": "GET"},
                {"role": "Reviewer", "endpoint_group": "edit", "method": "GET"},
                {"role": "Reviewer", "endpoint_group": "edit", "method": "POST"}
            ]
        }"#)?;

        let security = Builder::new()
            .anonymous_reader(true)
            .resource_policy(policy.clone())
            .build()
            .await?;
        assert!(!security.enforce(not_logged_in, "/item/1", "", "GET")?);
        assert!(security.enforce(Some("alice"), "/item/1", "", "GET")?);
        // this doesn't have the restriction the limited version poses
        assert!(security.enforce(Some("alice"), "/item/1", "edit", "GET")?);
        assert!(security.enforce(Some("alice"), "/item/1", "edit", "POST")?);
        assert!(security.enforce(Some("reviewer"), "/item/1", "", "GET")?);
        assert!(security.enforce(Some("reviewer"), "/item/1", "edit", "GET")?);
        assert!(security.enforce(Some("reviewer"), "/item/1", "edit", "POST")?);
        // this wasn't granted globally by the model, so this should have no effect.
        assert!(!security.enforce(Some("reviewer"), "/item/1", "grant", "POST")?);

        assert!(!security.enforce(Some("reviewer"), "/item/2", "", "GET")?);
        assert!(!security.enforce(Some("reviewer"), "/item/2", "edit", "GET")?);
        assert!(!security.enforce(Some("reviewer"), "/item/2", "edit", "POST")?);

        let security = Builder::new_limited()
            .anonymous_reader(true)
            .resource_policy(policy.clone())
            .build()
            .await?;
        assert!(!security.enforce(not_logged_in, "/item/1", "", "GET")?);
        assert!(security.enforce(Some("alice"), "/item/1", "", "GET")?);
        // the limited model restricts owners from submitting edits for
        // resources under review, but has no restrictions on viewing the
        // edit form.
        assert!(security.enforce(Some("alice"), "/item/1", "edit", "GET")?);
        assert!(!security.enforce(Some("alice"), "/item/1", "edit", "POST")?);
        assert!(security.enforce(Some("reviewer"), "/item/1", "", "GET")?);
        assert!(security.enforce(Some("reviewer"), "/item/1", "edit", "GET")?);
        assert!(security.enforce(Some("reviewer"), "/item/1", "edit", "POST")?);
        // this wasn't granted globally by the model, so this should have no effect.
        assert!(!security.enforce(Some("reviewer"), "/item/1", "grant", "POST")?);

        assert!(!security.enforce(Some("reviewer"), "/item/2", "", "GET")?);
        assert!(!security.enforce(Some("reviewer"), "/item/2", "edit", "GET")?);
        assert!(!security.enforce(Some("reviewer"), "/item/2", "edit", "POST")?);

        Ok(())
    }

    #[tokio::test]
    async fn policy_usage_reviewer_unconstrained() -> anyhow::Result<()> {
        // a rough approximation of a resource under review
        let policy: Policy = serde_json::from_str(r#"{
            "resource": "/item/1",
            "user_roles": [],
            "res_grants": [
                {"res": "/*", "agent": "reviewer", "role": "Reviewer"},
                {"res": "/item/1", "agent": "alice", "role": "Owner"}
            ],
            "role_permits": [
                {"role": "Reviewer", "endpoint_group": "", "method": "GET"},
                {"role": "Reviewer", "endpoint_group": "edit", "method": "GET"},
                {"role": "Reviewer", "endpoint_group": "edit", "method": "POST"}
            ]
        }"#)?;


        let mut security = Builder::new()
            .anonymous_reader(true)
            .resource_policy(policy.clone())
            .build()
            .await?;
        // the resource policy doesn't provide additional underlying policies, so this is
        // manually unconstrained to emulate the default policies defined for editor/manager
        security.attach_policy(Role::Reviewer, "/*", "", "GET").await?;
        security.attach_policy(Role::Reviewer, "/*", "edit", "GET").await?;
        security.attach_policy(Role::Reviewer, "/*", "edit", "POST").await?;

        assert!(security.enforce(Some("reviewer"), "/item/1", "", "GET")?);
        assert!(security.enforce(Some("reviewer"), "/item/1", "edit", "GET")?);
        assert!(security.enforce(Some("reviewer"), "/item/1", "edit", "POST")?);
        // note that /item/2 will also be permitted despite the resource only has /item/1
        assert!(security.enforce(Some("reviewer"), "/item/2", "", "GET")?);
        assert!(security.enforce(Some("reviewer"), "/item/2", "edit", "GET")?);
        assert!(security.enforce(Some("reviewer"), "/item/2", "edit", "POST")?);
        // anyway, the model may be possible to have further simplification.
        Ok(())
    }

    #[tokio::test]
    async fn policy_usage_published() -> anyhow::Result<()> {
        let not_logged_in: Option<&str> = None;
        let security = Builder::new()
            .anonymous_reader(true)
            .resource_policy(serde_json::from_str(r#"{
                "resource": "/item/1",
                "user_roles": [],
                "res_grants": [
                    {"res": "/*", "agent": "admin", "role": "Manager"},
                    {"res": "/item/1", "agent": "alice", "role": "Owner"}
                ],
                "role_permits": [
                    {"role": "Owner", "endpoint_group": "edit", "method": "GET"},
                    {"role": "Reader", "endpoint_group": "", "method": "GET"}
                ]
            }"#)?)
            .build()
            .await?;
        // a rough approximation of a published resource

        assert!(security.enforce(Some("admin"), "/item/1", "", "GET")?);
        assert!(security.enforce(Some("admin"), "/item/1", "edit", "GET")?);
        assert!(security.enforce(Some("admin"), "/item/1", "edit", "POST")?);
        assert!(security.enforce(Some("alice"), "/item/1", "", "GET")?);
        assert!(security.enforce(Some("alice"), "/item/1", "edit", "GET")?);
        assert!(security.enforce(Some("alice"), "/item/1", "edit", "POST")?);
        assert!(security.enforce(not_logged_in, "/item/1", "", "GET")?);
        assert!(!security.enforce(not_logged_in, "/item/1", "edit", "GET")?);
        assert!(!security.enforce(not_logged_in, "/item/1", "edit", "POST")?);

        Ok(())
    }

    #[tokio::test]
    async fn policy_usage_published_alt() -> anyhow::Result<()> {
        let not_logged_in: Option<&str> = None;
        let security = Builder::new_limited()
            .anonymous_reader(true)
            .resource_policy(serde_json::from_str(r#"{
                "resource": "/item/1",
                "user_roles": [],
                "res_grants": [
                    {"res": "/*", "agent": "admin", "role": "Manager"},
                    {"res": "/item/1", "agent": "alice", "role": "Owner"}
                ],
                "role_permits": [
                    {"role": "Owner", "endpoint_group": "edit", "method": "GET"},
                    {"role": "Reader", "endpoint_group": "", "method": "GET"}
                ]
            }"#)?)
            .build()
            .await?;
        // a rough approximation of a published resource

        assert!(security.enforce(Some("admin"), "/item/1", "", "GET")?);
        assert!(security.enforce(Some("admin"), "/item/1", "edit", "GET")?);
        assert!(security.enforce(Some("admin"), "/item/1", "edit", "POST")?);
        assert!(security.enforce(Some("alice"), "/item/1", "", "GET")?);
        assert!(security.enforce(Some("alice"), "/item/1", "edit", "GET")?);
        assert!(!security.enforce(Some("alice"), "/item/1", "edit", "POST")?);
        assert!(security.enforce(not_logged_in, "/item/1", "", "GET")?);
        assert!(!security.enforce(not_logged_in, "/item/1", "edit", "GET")?);
        assert!(!security.enforce(not_logged_in, "/item/1", "edit", "POST")?);

        Ok(())
    }
}
