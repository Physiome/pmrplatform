use sqlx_adapter::casbin::prelude::*;
use sqlx_adapter::casbin::Result;
use sqlx_adapter::SqlxAdapter;

/// The casbin model for pmrapp.
const DEFAULT_MODEL: &str = "\
[request_definition]
r = sub, res, ep, met

[policy_definition]
p = sub, res, ep, met

[role_definition]
g = _, _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = (g(r.sub, p.sub, r.res) || g(r.sub, p.sub, p.res)) && keyMatch2(r.res, p.res) && keyMatch(r.ep, p.ep) && r.met == p.met
";

// this matcher _doesn't_ work for the Rust version, so we just do another check for the None user even if there is Some(...) user.
// m = (g(r.sub, p.sub, r.res) || g(r.sub, p.sub, p.res) || g("-", p.sub, r.res) || g("-", p.sub, p.res)) && keyMatch2(r.res, p.res) && keyMatch(r.ep, p.ep) && r.met == p.met

/// The default policy for PMR
///
/// The fields are comma separated in the form of: role, route, endpoint group, HTTP method
///
/// role - the name of the role - it will be granted to users
/// route - the route to a given resource
/// endpoint group - the name for a given group of endpoints that share something in common
/// HTTP method - the permitted HTTP method associated with the policy
const DEFAULT_POLICIES: &str = "\
# managers can do everything
manager, /*, *, GET
manager, /*, *, POST

# readers have limited access
reader, /*, , GET
reader, /*, protocol, GET

# owners have everything, including granted access
owner, /*, , GET           # empty group signifies typical actions (e.g. view)
owner, /*, edit, GET       # edit signifies being able to edit content (e.g. exposure wizard)
owner, /*, edit, POST
owner, /*, grant, GET      # grant signifies being able to grant additional access
owner, /*, grant, POST
owner, /*, protocol, GET   # protocol signifies git clone/etc
owner, /*, protocol, POST

# editors have everything, can see grants but cannot grant additional access to others
editor, /*, , GET           # empty group signifies typical actions (e.g. view)
editor, /*, edit, GET
editor, /*, edit, POST
editor, /*, grant, GET
editor, /*, protocol, GET
editor, /*, protocol, POST
";

pub struct PmrRbac {
    pub enforcer: Enforcer,
}

impl PmrRbac {
    pub async fn open(db_url: &str) -> Result<Self> {
        let m = DefaultModel::from_str(DEFAULT_MODEL).await?;
        let a = SqlxAdapter::new(db_url, 8).await?;
        let enforcer = Enforcer::new(m, a).await?;
        log::info!("opened enforcer with backend {db_url}");
        Ok(Self { enforcer })
    }

    pub async fn new(db_url: &str) -> Result<Self> {
        let mut result = Self::open(db_url).await?;
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
        result.enforcer.add_named_policies("p", policies).await?;
        log::info!("new enforcer set up with {n} policies");
        Ok(result)
    }

    fn to_user(user: Option<impl AsRef<str> + std::fmt::Display>) -> String {
        user.map(|user| format!("u:{user}"))
            .unwrap_or("-".to_string())
    }

    /// Grant user specified role at resource.
    /// Creates the relevant casbin grouping policy.
    pub async fn grant(
        &mut self,
        user: Option<impl AsRef<str> + std::fmt::Display>,
        role: impl Into<String>,
        resource: impl Into<String>,
    ) -> Result<bool> {
        self.enforcer.add_named_grouping_policy("g", vec![
            Self::to_user(user),
            role.into(),
            resource.into(),
        ]).await
    }

    /// Reovkes user specified role at resource.
    /// Removes the relevant casbin grouping policy.
    pub async fn revoke(
        &mut self,
        user: Option<impl AsRef<str> + std::fmt::Display>,
        role: impl Into<String>,
        resource: impl Into<String>,
    ) -> Result<bool> {
        self.enforcer.remove_named_grouping_policy("g", vec![
            Self::to_user(user),
            role.into(),
            resource.into(),
        ]).await
    }

    /// Validates if the user accessing the path has the required rights.
    pub fn enforce(
        &self,
        user: Option<impl AsRef<str> + std::fmt::Display>,
        resource: impl AsRef<str>,
        endpoint_group: impl AsRef<str>,
        http_method: impl AsRef<str>,
    ) -> Result<bool> {
        Ok(
            self.enforcer.enforce((
                Self::to_user(user).as_str(),
                resource.as_ref(),
                endpoint_group.as_ref(),
                http_method.as_ref(),
            ))?
            ||
            self.enforcer.enforce((
                Self::to_user(None::<&str>).as_str(),
                resource.as_ref(),
                endpoint_group.as_ref(),
                http_method.as_ref(),
            ))?
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn demo() -> anyhow::Result<()> {
        let mut security = PmrRbac::new("sqlite::memory:").await?;
        let not_logged_in: Option<&str> = None;

        // admin account has access to every part of the application
        assert!(security.grant(Some("admin"), "manager", "/*").await?);
        // unregistered users are readers of every part of the application
        assert!(security.grant(not_logged_in, "reader", "/").await?);
        // alice is the owner of exposure 1
        assert!(security.grant(Some("alice"), "owner", "/exposure/1").await?);
        // bob is the owner of exposure 2
        assert!(security.grant(Some("bob"), "owner", "/exposure/2").await?);
        // cathy is the editor of exposure 2
        assert!(security.grant(Some("cathy"), "editor", "/exposure/2").await?);
        // make /exposure/1 public
        assert!(security.grant(not_logged_in, "reader", "/exposure/1").await?);

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

        // not logged in users can only view exposure 1
        assert!(security.enforce(not_logged_in, "/exposure/1", "", "GET")?);
        assert!(!security.enforce(not_logged_in, "/exposure/2", "", "GET")?);

        // not logged in users cannot edit/POST etc
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
