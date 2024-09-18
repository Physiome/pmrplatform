use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
pub enum State {
    // catch-all when infallable conversion is required
    #[default]
    Unknown,
    Private,
    Pending,
    Published,
    Expired,
}

/// Resource workflow state
///
/// Represents the access granted to a resource.  The grant associates
/// the user and the role for the resource.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct ResWorkflowState {
    pub res: String,
    pub state: State,
}

mod impls;
