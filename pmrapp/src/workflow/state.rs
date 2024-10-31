use serde::{
    Deserialize,
    Serialize,
};
use pmrcore::ac::{
    role::{
        Role,
        Roles,
    },
    workflow::State,
};
use std::{
    collections::HashMap,
    sync::LazyLock,
};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Transition {
    /// The target workflow state
    target: State,
    /// A description of the goal of this transition
    description: String,
    /// The roles that are permitted to use this transition
    roles: Roles,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Transitions(HashMap<State, Vec<Transition>>);

impl Default for Transitions {
    fn default() -> Self {
        Self(HashMap::from([
            (State::Private, vec![
                Transition {
                    target: State::Pending,
                    description: "Submit for publication".to_string(),
                    roles: Roles::from([
                        Role::Manager,
                        Role::Owner,
                        Role::Editor,
                    ]),
                },
                Transition {
                    target: State::Published,
                    description: "Publish".to_string(),
                    roles: Roles::from([
                        Role::Manager,
                        Role::Reviewer,
                    ]),
                },
            ]),
            (State::Pending, vec![
                Transition {
                    target: State::Private,
                    description: "Withdraw publication request".to_string(),
                    roles: Roles::from([
                        Role::Owner,
                        Role::Editor,
                    ]),
                },
                Transition {
                    target: State::Private,
                    description: "Reject publication request".to_string(),
                    roles: Roles::from([
                        Role::Manager,
                        Role::Reviewer,
                    ]),
                },
                Transition {
                    target: State::Published,
                    description: "Publish".to_string(),
                    roles: Roles::from([
                        Role::Manager,
                        Role::Reviewer,
                    ]),
                },
            ]),
            (State::Published, vec![
                Transition {
                    target: State::Private,
                    description: "Send back".to_string(),
                    roles: Roles::from([
                        Role::Manager,
                        Role::Editor,
                        Role::Reviewer,
                    ]),
                },
                Transition {
                    target: State::Expired,
                    description: "Expire".to_string(),
                    roles: Roles::from([
                        Role::Manager,
                        Role::Editor,
                        Role::Reviewer,
                    ]),
                },
            ]),
            (State::Published, vec![
                Transition {
                    target: State::Private,
                    description: "Send back".to_string(),
                    roles: Roles::from([
                        Role::Manager,
                        Role::Editor,
                        Role::Reviewer,
                    ]),
                },
                Transition {
                    target: State::Published,
                    description: "Restore".to_string(),
                    roles: Roles::from([
                        Role::Manager,
                        Role::Editor,
                        Role::Reviewer,
                    ]),
                },
            ]),
        ]))
    }
}

pub static TRANSITIONS: LazyLock<Transitions> = LazyLock::new(|| {
    Transitions::default()
});
