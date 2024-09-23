use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash,
        PasswordHasher,
        PasswordVerifier,
        SaltString,
    },
    Argon2,
};
use pmrcore::platform::ACPlatform;
use std::sync::Arc;

use crate::{
    error::Error,
    user::User,
};

pub struct Platform {
    platform: Arc<dyn ACPlatform>,
}

impl Platform {
    pub fn new(
        platform: impl ACPlatform + 'static,
    ) -> Self {
        let platform = Arc::new(platform);
        Self { platform }
    }

    pub async fn create_user<'a>(
        &'a self,
        name: &str,
    ) -> Result<User, Error> {
        let id = self.platform.add_user(name).await?;
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

    /// Set a user's password using the user's id.
    pub async fn set_user_id_password(
        &self,
        id: i64,
        password: &str,
    ) -> Result<(), Error> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)?
            .to_string();
        self.platform.store_user_password(id, &password_hash).await?;
        Ok(())
    }

    pub async fn validate_user_id_password(
        &self,
        id: i64,
        password: &str,
    ) -> Result<bool, Error> {
        let password_hash = self.platform.get_user_password(id).await?;
        let parsed_hash = PasswordHash::new(&password_hash)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
        )
    }
}
