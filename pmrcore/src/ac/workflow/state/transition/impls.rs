use enumset::EnumSet;
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

impl StateTransitions {
    pub fn transitions_for(&self, state: State, roles: Roles) -> Vec<&Transition> {
        self.0.get(&state)
            .map(|transitions| {
                transitions.iter()
                    .filter_map(|transition| (!(transition.roles & roles).0.is_empty())
                        .then_some(transition))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| Vec::new())
    }

    pub fn validate(&self, roles: Roles, from: State, to: State) -> bool {
        self.0.get(&from)
            .map(|transitions| {
                transitions.iter()
                    .filter_map(|transition| (!(transition.roles & roles).0.is_empty())
                        .then_some(transition.target))
                    .collect::<EnumSet<_>>() & to == to
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke() {
        let st = StateTransitions::default();
        assert_eq!(st.transitions_for(State::Unknown, [Role::Undefined].into()).len(), 0);
        assert_eq!(st.transitions_for(State::Private, [Role::Reader].into()).len(), 0);
        assert_eq!(st.transitions_for(State::Private, [Role::Owner].into()).len(), 1);
        assert_eq!(st.transitions_for(State::Private, [Role::Manager].into()).len(), 2);

        assert!(!st.validate([Role::Reader].into(), State::Private, State::Pending));
        assert!(st.validate([Role::Owner].into(), State::Private, State::Pending));
        assert!(!st.validate([Role::Owner].into(), State::Private, State::Published));
        assert!(st.validate([Role::Reader, Role::Manager].into(), State::Private, State::Published));

        let values = st.transitions_for(State::Pending, [Role::Owner].into())
            .into_iter()
            .map(|Transition { target, description, .. }| (*target, description.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(&values, &[
            (State::Private, "Withdraw publication request"),
        ]);

        let values = st.transitions_for(State::Pending, [Role::Manager].into())
            .into_iter()
            .map(|Transition { target, description, .. }| (*target, description.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(&values, &[
            (State::Private, "Reject publication request"),
            (State::Published, "Publish"),
        ]);

        let values = st.transitions_for(State::Pending, [Role::Owner, Role::Manager].into())
            .into_iter()
            .map(|Transition { target, description, .. }| (*target, description.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(&values, &[
            (State::Private, "Withdraw publication request"),
            (State::Private, "Reject publication request"),
            (State::Published, "Publish"),
        ]);
    }
}
