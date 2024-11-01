use pmrcore::ac::workflow::state::transition::StateTransitions;
use std::sync::LazyLock;

// TODO figure out how to best share this in a configurable manner
// between server and client.  For now have this be compiled directly
// into both binaries.
pub static TRANSITIONS: LazyLock<StateTransitions> = LazyLock::new(|| {
    StateTransitions::default()
});
