use pmrcore::{
    ac::{
        agent::Agent,
        genpolicy::Policy,
        role::Role,
        session::{
            self,
            SessionFactory,
            SessionToken,
        },
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
    session::Session,
};

use super::*;

impl Builder {
    pub fn new() -> Self {
        Self {
            pmrrbac_builder: PmrRbacBuilder::new(),
            .. Default::default()
        }
    }

    pub fn ac_platform(mut self, val: impl ACPlatform + 'static) -> Self {
        self.ac_platform = Some(Box::new(val));
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

    pub fn session_factory(mut self, val: SessionFactory) -> Self {
        self.session_factory = val;
        self
    }

    pub fn build(self) -> Platform {
        Platform(Arc::new(PlatformInner {
            ac_platform: self.ac_platform
                .expect("missing required argument ac_platform"),
            password_autopurge: self.password_autopurge,
            pmrrbac_builder: self.pmrrbac_builder,
            session_factory: self.session_factory,
        }))
    }
}

impl Platform {
    pub(crate) fn ac_platform(&self) -> &dyn ACPlatform {
        self.0.ac_platform.as_ref()
    }
}

// User management.
impl Platform {
    pub async fn create_user(
        &self,
        name: &str,
    ) -> Result<User, Error> {
        let id = self.0.ac_platform.add_user(name).await?;
        self.force_user_id_password(id, Password::New).await?;
        Ok(self.get_user(id).await?
            .expect("the user should have been created"))
    }

    // TODO eventually this might go away - the adminstrator will be using this
    // platform directly and rarely will have to go through the user object, as
    // the user object should typically be acquired as part of the session for
    // the actual agent associated with that session.
    pub async fn get_user(
        &self,
        id: i64,
    ) -> Result<Option<User>, Error> {
        let user = self.0.ac_platform.get_user_by_id(id).await?
            .map(|user| User::new(self.clone(), user));
        Ok(user)
    }

    pub async fn authenticate_user(
        &self,
        login: &str,
        password: &str,
    ) -> Result<User, Error> {
        // TODO login can be email also
        let user = self.0.ac_platform.get_user_by_name(login).await?
            .ok_or(AuthenticationError::UnknownUser)?;
        self.verify_user_id_password(user.id, password).await?;
        Ok(User::new(self.clone(), user))
    }

    pub async fn login_status(
        &self,
        login: &str,
    ) -> Result<(user::User, PasswordStatus), Error> {
        // TODO login can be email also
        // TODO should report this error better, e.g. need an enum for user not exist
        let user = self.0.ac_platform.get_user_by_name(login).await?
            .ok_or(AuthenticationError::UnknownUser)?;
        let result = self.0.ac_platform.get_user_password(user.id).await;
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
        let result = self.0.ac_platform.get_user_password(id).await;
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
        let result = self.0.ac_platform.get_user_password(id).await;
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
        if self.0.password_autopurge {
            self.0.ac_platform.purge_user_passwords(id).await?;
        }
        self.0.ac_platform.store_user_password(id, &password_hash).await?;
        Ok(())
    }
}

// Agent Policy management

impl Platform {
    pub async fn grant_role_to_user(
        &self,
        user: impl Into<user::User>,
        role: Role,
    ) -> Result<bool, Error> {
        Ok(self.0.ac_platform.grant_role_to_user(
            &user.into(),
            role
        ).await?)
    }

    pub async fn revoke_role_from_user(
        &self,
        user: impl Into<user::User>,
        role: Role,
    ) -> Result<bool, Error> {
        Ok(self.0.ac_platform.revoke_role_from_user(
            &user.into(),
            role,
        ).await?)
    }

    pub async fn res_grant_role_to_agent(
        &self,
        res: &str,
        agent: impl Into<Agent>,
        role: Role,
    ) -> Result<bool, Error> {
        Ok(self.0.ac_platform.res_grant_role_to_agent(
            res,
            &agent.into(),
            role
        ).await?)
    }

    pub async fn res_revoke_role_from_agent(
        &self,
        res: &str,
        agent: impl Into<Agent>,
        role: Role,
    ) -> Result<bool, Error> {
        Ok(self.0.ac_platform.res_revoke_role_from_agent(
            res,
            &agent.into(),
            role,
        ).await?)
    }

    pub async fn get_res_grants_for_res(
        &self,
        res: &str,
    ) -> Result<Vec<(Agent, Vec<Role>)>, Error> {
        Ok(self.0.ac_platform.get_res_grants_for_res(
            res,
        ).await?)
    }

    pub async fn get_res_grants_for_agent(
        &self,
        agent: &Agent,
    ) -> Result<Vec<(String, Vec<Role>)>, Error> {
        Ok(self.0.ac_platform.get_res_grants_for_agent(
            agent,
        ).await?)
    }

    pub async fn assign_policy_to_wf_state(
        &self,
        wf_state: State,
        role: Role,
        endpoint_group: &str,
        method: &str,
    ) -> Result<(), Error> {
        Ok(self.0.ac_platform.assign_policy_to_wf_state(
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
        Ok(self.0.ac_platform.remove_policy_from_wf_state(
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
        Ok(self.0.ac_platform.get_wf_state_for_res(
            res,
        ).await?)
    }

    pub async fn set_wf_state_for_res(
        &self,
        res: &str,
        wf_state: State,
    ) -> Result<(), Error> {
        Ok(self.0.ac_platform.set_wf_state_for_res(
            res,
            wf_state,
        ).await?)
    }

    pub async fn generate_policy_for_agent_res(
        &self,
        agent: &Agent,
        res: String,
    ) -> Result<Policy, Error> {
        Ok(self.0.ac_platform.generate_policy_for_agent_res(
            agent,
            res,
        ).await?)
    }
}

// Session management

impl Platform {
    pub async fn new_user_session(
        &self,
        user: User,
        origin: String,
    ) -> Result<Session, Error> {
        // this wouldn't work in stable as trait upcasting is nightly
        // let session = Session::new(
        //     &self,
        //     SessionBackend::save_session(
        //             self.0.ac_platform.as_ref(),
        //             &self.0.session_factory,
        //             user.id(),
        //             origin,
        //         )
        //         .await?,
        //     user,
        // );
        // ... so just inline the trivial `new_user_session` here, at
        // least until this is fully stablized.
        let session = self.0.session_factory.create(user.id(), origin);
        self.0.ac_platform.save_session(&session).await?;
        let session = Session::new(self.clone(), session, user);
        Ok(session)
    }

    pub async fn load_session(
        &self,
        token: SessionToken,
    ) -> Result<Session, Error> {
        let session = self.0.ac_platform.load_session(token).await?;
        let user = self.get_user(session.user_id).await?
            .ok_or(AuthenticationError::UnknownUser)?;
        Ok(Session::new(
            self.clone(),
            session,
            user,
        ))
    }

    /// Simply return a list of sessions without the token for the user_id
    pub async fn get_user_sessions(
        &self,
        user_id: i64,
    ) -> Result<Vec<session::Session>, Error> {
        Ok(self.0.ac_platform.get_user_sessions(user_id).await?)
    }

    /// Logout all sessions associated with the user_id.
    pub async fn logout_user(
        &self,
        user_id: i64,
    ) -> Result<(), Error> {
        Ok(self.0.ac_platform.purge_user_sessions(user_id, None).await?)
    }
}

// Login methods

impl Platform {
    pub async fn authenticate_user_login(
        &self,
        login: &str,
        password: &str,
        origin: String,
    ) -> Result<Session, Error> {
        let user = self.authenticate_user(login, password).await?;
        let session = self.new_user_session(user, origin).await?;
        Ok(session)
    }

    // TODO login via some generated token (e.g. account password reset
    // token generated via new user or password reset).
    // pub async fn authenticate_user_token(
    //     &self,
    //     login: &str,
    //     token: &str,
    //     origin: String,
    // ) -> Result<Session, Error> {
    //
    // When adapting for axum-login, its `Credentials` will be a enum listing
    // all possible login credentials from above.
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
        Ok(self.0.pmrrbac_builder
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
