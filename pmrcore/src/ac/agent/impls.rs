use crate::ac::user::User;
use super::Agent;

impl From<User> for Agent {
    fn from(user: User) -> Agent {
        Agent::User(user)
    }
}

impl From<&Agent> for Option<i64> {
    fn from(agent: &Agent) -> Self {
        match agent {
            Agent::Anonymous => None,
            Agent::User(User { id, .. }) => Some(*id),
        }
    }
}

impl From<Agent> for Option<String> {
    fn from(agent: Agent) -> Self {
        match agent {
            Agent::Anonymous => None,
            Agent::User(User { name, .. }) => Some(name),
        }
    }
}
