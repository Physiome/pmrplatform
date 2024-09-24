use pmrcore::ac::user;

use crate::Platform;

pub struct User<'a> {
    platform: &'a Platform,
    user: user::User,
}

mod impls;
