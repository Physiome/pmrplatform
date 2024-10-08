use pmrcore::{
    ac::{
        agent::Agent,
        genpolicy::Policy,
        role::Role,
        user,
        workflow::State,
    },
    platform::ACPlatform
};
use pmrrbac::Builder as PmrRbacBuilder;
use std::sync::Arc;

use crate::{
    error::{
        AuthenticationError,
        Error,
        PasswordError,
    },
    user::User,
    password::{
        Password,
        PasswordStatus,
    },
};

#[derive(Clone, Default)]
pub struct Builder {
    // platform
    ac_platform: Option<Arc<dyn ACPlatform>>,
    // automatically purges all but the most recent passwords
    password_autopurge: bool,
    pmrrbac_builder: PmrRbacBuilder,
}

pub struct Platform {
    ac_platform: Arc<dyn ACPlatform>,
    password_autopurge: bool,
    pmrrbac_builder: PmrRbacBuilder,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            pmrrbac_builder: PmrRbacBuilder::new(),
            .. Default::default()
        }
    }

    pub fn ac_platform(mut self, val: impl ACPlatform + 'static) -> Self {
        self.ac_platform = Some(Arc::new(val));
        self
    }

    pub fn password_autopurge(mut self, val: bool) -> Self {
        self.password_autopurge = val;
        self
    }

    pub fn pmrrbac_builder(mut self, val: PmrRbacBuilder) -> Self {
        self.pmrrbac_builder = val;
        self
    }

    pub fn build(self) -> Platform {
        Platform {
            ac_platform: self.ac_platform.expect("missing required argument ac_platform"),
            password_autopurge: self.password_autopurge,
            pmrrbac_builder: self.pmrrbac_builder
        }
    }
}

impl Platform {
    pub fn new(
        ac_platform: impl ACPlatform + 'static,
        password_autopurge: bool,
        pmrrbac_builder: PmrRbacBuilder,
    ) -> Self {
        let ac_platform = Arc::new(ac_platform);
        Self {
            ac_platform,
            password_autopurge,
            pmrrbac_builder,
        }
    }
}

// User management.
impl<'a> Platform {
    pub async fn create_user(
        &'a self,
        name: &str,
    ) -> Result<User, Error> {
        let id = self.ac_platform.add_user(name).await?;
        self.force_user_id_password(id, Password::New).await?;
        self.get_user(id).await
    }

    // TODO eventually this might go away - the adminstrator will be using this
    // platform directly and rarely will have to go through the user object, as
    // the user object should typically be acquired as part of the session for
    // the actual agent associated with that session.
    pub async fn get_user(
        &'a self,
        id: i64,
    ) -> Result<User, Error> {
        let user = self.ac_platform.get_user_by_id(id).await?;
        Ok(User::new(self, user))
    }

    pub async fn authenticate_user(
        &'a self,
        login: &str,
        password: &str,
    ) -> Result<User<'a>, Error> {
        // TODO login can be email also
        let user = self.ac_platform.get_user_by_name(login).await?;
        self.verify_user_id_password(user.id, password).await?;
        Ok(User::new(self, user))
    }

    pub async fn login_status(
        &self,
        login: &str,
    ) -> Result<(user::User, PasswordStatus), Error> {
        // TODO login can be email also
        // TODO should report this error better, e.g. need an enum for user not exist
        let user = self.ac_platform.get_user_by_name(login).await?;
        let result = self.ac_platform.get_user_password(user.id).await;
        let password = result
            .as_deref()
            .map(Password::from_database)
            .unwrap_or(Password::Misconfigured);
        Ok((user, password.into()))
    }
}

// Password management

impl Platform {
    /// Set a user's password using the user's id using the provided
    /// `&str` if a new password may be set.  This will only set the
    /// desired password iff the stored password is New or Reset.
    pub async fn new_user_id_password(
        &self,
        id: i64,
        password: &str,
    ) -> Result<(), Error> {
        let result = self.ac_platform.get_user_password(id).await;
        let stored_password = result
            .as_deref()
            .map(Password::from_database)
            .unwrap_or(Password::Misconfigured);
        match stored_password {
            Password::New | Password::Reset =>
                self.force_user_id_password(
                    id,
                    Password::new(password)
                ).await,
            Password::Hash(_) => Err(PasswordError::Existing)?,
            Password::Restricted => Err(AuthenticationError::Restricted)?,
            _ => Err(Error::Misconfiguration),
        }
    }

    /// This verify the incoming string as a raw password against the
    /// hashed version stored in the database.
    pub async fn verify_user_id_password(
        &self,
        id: i64,
        password: &str,
    ) -> Result<(), Error> {
        let result = self.ac_platform.get_user_password(id).await;
        let stored_password = result
            .as_deref()
            .map(Password::from_database)
            .unwrap_or(Password::Misconfigured);
        Ok(stored_password.verify(&Password::new(password))?)
    }

    /// Forcibly set a user's password using the user's id using the
    /// provided `Password`
    pub async fn force_user_id_password(
        &self,
        id: i64,
        password: Password<'_>,
    ) -> Result<(), Error> {
        let password_hash = password.to_database()?;
        if self.password_autopurge {
            self.ac_platform.purge_user_passwords(id).await?;
        }
        self.ac_platform.store_user_password(id, &password_hash).await?;
        Ok(())
    }
}

// Agent Policy management

impl Platform {
    pub async fn grant_role_to_user(
        &self,
        user: impl Into<user::User>,
        role: Role,
    ) -> Result<(), Error> {
        Ok(self.ac_platform.grant_role_to_user(
            &user.into(),
            role
        ).await?)
    }

    pub async fn revoke_role_from_user(
        &self,
        user: impl Into<user::User>,
        role: Role,
    ) -> Result<(), Error> {
        Ok(self.ac_platform.revoke_role_from_user(
            &user.into(),
            role,
        ).await?)
    }

    pub async fn grant_res_role_to_agent(
        &self,
        res: &str,
        agent: impl Into<Agent>,
        role: Role,
    ) -> Result<(), Error> {
        Ok(self.ac_platform.grant_res_role_to_agent(
            res,
            &agent.into(),
            role
        ).await?)
    }

    pub async fn revoke_res_role_from_agent(
        &self,
        res: &str,
        agent: impl Into<Agent>,
        role: Role,
    ) -> Result<(), Error> {
        Ok(self.ac_platform.revoke_res_role_from_agent(
            res,
            &agent.into(),
            role,
        ).await?)
    }

    pub async fn get_res_grants(
        &self,
        res: &str,
    ) -> Result<Vec<(Agent, Role)>, Error> {
        Ok(self.ac_platform.get_res_grants(
            res,
        ).await?)
    }

    pub async fn assign_policy_to_wf_state(
        &self,
        wf_state: State,
        role: Role,
        endpoint_group: &str,
        method: &str,
    ) -> Result<(), Error> {
        Ok(self.ac_platform.assign_policy_to_wf_state(
            wf_state,
            role,
            endpoint_group,
            method,
        ).await?)
    }

    pub async fn remove_policy_from_wf_state(
        &self,
        wf_state: State,
        role: Role,
        endpoint_group: &str,
        method: &str,
    ) -> Result<(), Error> {
        Ok(self.ac_platform.remove_policy_from_wf_state(
            wf_state,
            role,
            endpoint_group,
            method,
        ).await?)
    }
}

// Resource management

impl Platform {
    pub async fn get_wf_state_for_res(
        &self,
        res: &str,
    ) -> Result<State, Error> {
        Ok(self.ac_platform.get_wf_state_for_res(
            res,
        ).await?)
    }

    pub async fn set_wf_state_for_res(
        &self,
        res: &str,
        wf_state: State,
    ) -> Result<(), Error> {
        Ok(self.ac_platform.set_wf_state_for_res(
            res,
            wf_state,
        ).await?)
    }

    pub async fn generate_policy_for_agent_res(
        &self,
        agent: &Agent,
        res: String,
    ) -> Result<Policy, Error> {
        Ok(self.ac_platform.generate_policy_for_agent_res(
            agent,
            res,
        ).await?)
    }
}

// Enforcement

impl Platform {
    pub async fn enforce(
        &self,
        agent: impl Into<Agent>,
        res: impl AsRef<str> + ToString,
        endpoint_group: impl AsRef<str>,
        http_method: &str,
    ) -> Result<bool, Error> {
        let agent = agent.into();
        Ok(self.pmrrbac_builder
            .build_with_resource_policy(
                self.generate_policy_for_agent_res(
                    &agent,
                    res.to_string(),
                ).await?,
            )
            .await?
            .enforce(
                <Agent as Into<Option<String>>>::into(agent),
                res,
                endpoint_group,
                http_method,
            )?)
    }
}
