use serde::{
    Deserialize,
    Serialize,
};
use std::collections::HashMap;

use crate::ac::{
    role::{
        Role,
        Roles,
    },
    workflow::State,
};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Transition {
    /// The target workflow state
    pub target: State,
    /// A description of the goal of this transition
    pub description: String,
    /// The roles that are permitted to use this transition
    pub roles: Roles,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct StateTransitions(HashMap<State, Vec<Transition>>);

mod impls;
