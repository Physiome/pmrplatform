use pmrcore::ac::user;

use crate::{
    Platform,
    error::{
        Error,
        PasswordError,
    },
};

pub struct User<'a> {
    platform: &'a Platform,
    user: user::User,
}

impl<'a> User<'a> {
    pub(crate) fn new(
        platform: &'a Platform,
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

    pub fn name(&'a self) -> &'a str {
        self.user.name.as_ref()
    }

    pub async fn update_password(
        &self,
        // TODO old password may be optional (e.g. new user or forgot password), need to address this
        // TODO maybe need to decide what kind of raw string tokens may be in the password field to
        // denote this?
        old_password: &str,
        new_password: &str,
        new_password_confirm: &str,
    ) -> Result<(), Error> {
        self.platform
            .validate_user_id_password(
                self.user.id,
                old_password,
            )
            .await?
            .then_some(())
            .ok_or(PasswordError::WrongPassword)?;
        (new_password == new_password_confirm)
            .then_some(())
            .ok_or(PasswordError::MismatchedPassword)?;
        Ok(self.platform
            .set_user_id_password(
                self.user.id,
                new_password,
            )
            .await?
        )
    }
}
