use pmrcore::ac::workflow::state::transition::StateTransitions;
use std::sync::LazyLock;

pub static TRANSITIONS: LazyLock<StateTransitions> = LazyLock::new(|| {
    StateTransitions::default()
});
