use serde::{Deserialize, Serialize};

pub mod state;
pub use state::State;

/// Resource workflow state
///
/// Represents the access granted to a resource.  The grant associates
/// the user and the role for the resource.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ResWorkflowState {
    pub res: String,
    pub state: State,
}
