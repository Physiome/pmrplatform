use pmrcore::ac::{
    agent::Agent,
    user,
};

use crate::{
    error::{
        Error,
        PasswordError,
    },
    password::Password,
    Platform,
};
use super::User;

impl User {
    pub(crate) fn new(
        platform: Platform,
        user: user::User,
    ) -> Self {
        Self {
            platform,
            user,
        }
    }

    pub fn id(&self) -> i64 {
        self.user.id
    }

    pub fn name(&self) -> &str {
        self.user.name.as_ref()
    }

    pub fn clone_inner(&self) -> user::User {
        self.user.clone()
    }

    pub fn into_inner(self) -> user::User {
        self.user
    }

    pub async fn update_password(
        &self,
        old_password: &str,
        new_password: &str,
        new_password_confirm: &str,
    ) -> Result<(), Error> {
        (new_password == new_password_confirm)
            .then_some(())
            .ok_or(PasswordError::Mismatched)?;
        self.platform
            .verify_user_id_password(
                self.user.id,
                old_password,
            )
            .await?;
        Ok(self.platform
            .force_user_id_password(
                self.user.id,
                Password::new(new_password),
            )
            .await?
        )
    }

    pub async fn reset_password(
        &self,
        new_password: &str,
        new_password_confirm: &str,
    ) -> Result<(), Error> {
        (new_password == new_password_confirm)
            .then_some(())
            .ok_or(PasswordError::Mismatched)?;
        Ok(self.platform
            .new_user_id_password(
                self.user.id,
                new_password,
            )
            .await?
        )
    }
}

impl From<&User> for Agent {
    fn from(user: &User) -> Self {
        user.clone_inner().into()
    }
}

impl From<User> for Agent {
    fn from(user: User) -> Self {
        user.into_inner().into()
    }
}

impl From<&User> for user::User {
    fn from(user: &User) -> Self {
        user.clone_inner()
    }
}

impl From<User> for user::User {
    fn from(user: User) -> Self {
        user.into_inner()
    }
}
