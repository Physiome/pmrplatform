use super::*;

impl Default for StateTransitions {
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
            (State::Expired, vec![
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
