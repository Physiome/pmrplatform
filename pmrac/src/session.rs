use pmrcore::ac::session;

use crate::{
    Platform,
    user::User,
};

pub struct Session<'a> {
    platform: &'a Platform,
    session: session::Session,
    user: User<'a>,
}

mod impls;
