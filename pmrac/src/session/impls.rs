use pmrcore::ac::session;

use crate::{
    error::Error,
    platform::Platform,
    user::User,
};
use super::Session;

impl<'a> Session<'a> {
    pub(crate) fn new(
        platform: &'a Platform,
        session: session::Session,
        user: User<'a>,
    ) -> Self {
        Self {
            platform,
            session,
            user,
        }
    }

    pub fn user(&self) -> &User<'a> {
        &self.user
    }

    // access to every field, which may or may not be what we want.
    pub fn session(&self) -> &session::Session {
        &self.session
    }

    // consider making the argument `self` to consume, and not worry
    // about dealing with the timestamp at all?
    pub async fn save(&self) -> Result<i64, Error> {
        Ok(self.platform
            .ac_platform()
            .save_session(&self.session)
            .await?)
    }

    /// Logout this session.
    pub async fn logout(self) -> Result<(), Error> {
        Ok(self.platform
            .ac_platform()
            .purge_session(self.session.token)
            .await?)
    }

    /// Logout all other sessions assoicated with the user.
    pub async fn logout_others(&self) -> Result<(), Error> {
        Ok(self.platform
            .ac_platform()
            .purge_user_sessions(
                self.user().id(),
                Some(self.session.token),
            )
            .await?)
    }
}
