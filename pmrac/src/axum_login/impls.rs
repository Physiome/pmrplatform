use crate::{
    error::Error,
    platform::Platform,
    session::Session,
};
use super::*;

use ::axum_login::{
    AuthnBackend,
    UserId,
};

impl AuthnBackend for Platform {
    type User = Session;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(
        &self,
        credentials: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        match credentials.authorization {
            Authorization::LoginPassword(login, password) => {
                let session = self.authenticate_user_login(
                    &login,
                    &password,
                    credentials.origin,
                ).await?;
                Ok(Some(session))
            }
            _ => unimplemented!(),
        }
    }

    async fn get_user(
        &self,
        session_token: &UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
        Ok(Some(self.load_session(*session_token).await?))
    }
}

mod manager;
