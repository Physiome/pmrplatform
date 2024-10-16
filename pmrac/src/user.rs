use pmrcore::ac::user;

use crate::Platform;

#[derive(Clone)]
pub struct User {
    platform: Platform,
    user: user::User,
}

mod impls;
