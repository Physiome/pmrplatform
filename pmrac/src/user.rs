use pmrcore::ac::user;

use crate::platform::Platform;

pub struct User<'a> {
    platform: &'a Platform,
    user: user::User,
}

mod impls;
