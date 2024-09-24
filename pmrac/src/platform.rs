use pmrcore::platform::ACPlatform;
use std::sync::Arc;

use crate::{
    error::{
        AuthenticationError,
        Error,
        PasswordError,
    },
    user::User,
    password::Password,
};

#[derive(Clone, Default)]
pub struct Builder {
    // platform
    platform: Option<Arc<dyn ACPlatform>>,
    // automatically purges all but the most recent passwords
    password_autopurge: bool,
}

pub struct Platform {
    platform: Arc<dyn ACPlatform>,
    password_autopurge: bool,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn platform(mut self, val: impl ACPlatform + 'static) -> Self {
        self.platform = Some(Arc::new(val));
        self
    }

    pub fn password_autopurge(mut self, val: bool) -> Self {
        self.password_autopurge = val;
        self
    }

    pub fn build(self) -> Platform {
        Platform {
            platform: self.platform.expect("missing required argument platform"),
            password_autopurge: self.password_autopurge,
        }
    }
}

impl Platform {
    pub fn new(
        platform: impl ACPlatform + 'static,
        password_autopurge: bool,
    ) -> Self {
        let platform = Arc::new(platform);
        Self { platform, password_autopurge }
    }

    pub async fn create_user<'a>(
        &'a self,
        name: &str,
    ) -> Result<User, Error> {
        let id = self.platform.add_user(name).await?;
        self.force_user_id_password(id, Password::New).await?;
        self.get_user(id).await
    }

    // TODO eventually this might go away - the adminstrator will be using this
    // platform directly and rarely will have to go through the user object, as
    // the user object should typically be acquired as part of the session for
    // the actual agent associated with that session.
    pub async fn get_user<'a>(
        &'a self,
        id: i64,
    ) -> Result<User, Error> {
        let user = self.platform.get_user_by_id(id).await?;
        Ok(User::new(self, user))
    }

    /// Set a user's password using the user's id using the provided
    /// `&str` if a new password may be set.  This will only set the
    /// desired password iff the stored password is New or Reset.
    pub async fn new_user_id_password(
        &self,
        id: i64,
        password: &str,
    ) -> Result<(), Error> {
        let result = self.platform.get_user_password(id).await;
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
        let result = self.platform.get_user_password(id).await;
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
            self.platform.purge_user_passwords(id).await?;
        }
        self.platform.store_user_password(id, &password_hash).await?;
        Ok(())
    }
}
