use pmrcore::ac::session;

use crate::{
    platform::Platform,
    user::User,
};

pub struct Session<'a> {
    platform: &'a Platform,
    session: session::Session,
    user: User<'a>,
}

mod impls;
