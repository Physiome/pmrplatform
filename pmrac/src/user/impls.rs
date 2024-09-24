use pmrcore::ac::user;

use crate::{
    Platform,
    error::{
        Error,
        PasswordError,
    },
    password::Password,
};
use super::User;

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
