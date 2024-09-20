use pmrcore::ac::role::Role;

use casbin::prelude::*;
use casbin::Result;

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
Owner, /*, edit, POST
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

pub struct PmrRbac {
    enforcer: Enforcer,
}

impl PmrRbac {
    pub async fn new() -> Result<Self> {
        let m = DefaultModel::from_str(DEFAULT_MODEL).await?;
        let a = MemoryAdapter::default();
        let mut enforcer = Enforcer::new(m, a).await?;
        let policies = DEFAULT_POLICIES.lines()
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
        Ok(Self { enforcer })
    }

    fn to_agent(agent: Option<impl AsRef<str> + std::fmt::Display>) -> String {
        agent.map(|agent| format!("u:{agent}"))
            .unwrap_or("-".to_string())
    }

    /// Create agent assigns the reader role to the agent through the
    /// second grouping policy
    pub async fn create_agent(
        &mut self,
        agent: Option<impl AsRef<str> + std::fmt::Display>,
    ) -> Result<bool> {
        self.enforcer.add_named_grouping_policy("g2", vec![
            Self::to_agent(agent),
            Role::Reader.into(),
        ]).await
    }

    /// Remove agent removes the reader role for the second grouping.
    pub async fn remove_agent(
        &mut self,
        agent: impl AsRef<str> + std::fmt::Display,
    ) -> Result<bool> {
        self.enforcer.remove_named_grouping_policy("g2", vec![
            Self::to_agent(Some(agent)),
            Role::Reader.into(),
        ]).await
    }

    /// Grant agent specified role at resource.
    /// Creates the relevant casbin grouping policy.
    pub async fn grant(
        &mut self,
        agent: Option<impl AsRef<str> + std::fmt::Display>,
        role: impl Into<String>,
        resource: impl Into<String>,
    ) -> Result<bool> {
        self.enforcer.add_named_grouping_policy("g", vec![
            Self::to_agent(agent),
            role.into(),
            resource.into(),
        ]).await
    }

    /// Revokes agent specified role at resource.
    /// Removes the relevant casbin grouping policy.
    pub async fn revoke(
        &mut self,
        agent: Option<impl AsRef<str> + std::fmt::Display>,
        role: impl Into<String>,
        resource: impl Into<String>,
    ) -> Result<bool> {
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
    ) -> Result<bool> {
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
    ) -> Result<bool> {
        self.enforcer.remove_named_policy("p", vec![
            role.into(),
            resource.into(),
            endpoint_group.into(),
            http_method.into(),
        ]).await
    }

    /// Validates if the agent accessing the path has the required rights.
    pub fn enforce(
        &self,
        agent: Option<impl AsRef<str> + std::fmt::Display>,
        resource: impl AsRef<str>,
        endpoint_group: impl AsRef<str>,
        http_method: impl AsRef<str>,
    ) -> Result<bool> {
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
    async fn demo() -> anyhow::Result<()> {
        let mut security = PmrRbac::new().await?;
        let not_logged_in: Option<&str> = None;

        // admin account has access to every part of the application
        assert!(security.grant(Some("admin"), Role::Manager, "/*").await?);
        // alice is the owner of exposure 1
        assert!(security.create_agent(Some("alice")).await?);
        assert!(security.grant(Some("alice"), Role::Owner, "/exposure/1").await?);
        // bob is the owner of exposure 2
        assert!(security.create_agent(Some("bob")).await?);
        assert!(security.grant(Some("bob"), Role::Owner, "/exposure/2").await?);
        // cathy is the editor of exposure 2
        assert!(security.grant(Some("cathy"), Role::Editor, "/exposure/2").await?);
        // create the anonymous agent also
        assert!(security.create_agent(not_logged_in).await?);
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
}
