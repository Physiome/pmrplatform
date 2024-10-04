use std::fmt::{
    Display,
    Formatter,
    Result,
};

use crate::ac::user::User;
use super::Agent;

impl Display for Agent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}",
            match self {
                Agent::Anonymous => format!("<Agent:Anonymous>"),
                Agent::User(User { name, .. }) => format!("<User:{name}>"),
            }
        )
    }
}
