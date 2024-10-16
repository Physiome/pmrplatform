use pmrcore::ac::session;

use crate::{
    Platform,
    user::User,
};

#[derive(Clone)]
pub struct Session {
    platform: Platform,
    session: session::Session,
    user: User,
}

mod impls;
