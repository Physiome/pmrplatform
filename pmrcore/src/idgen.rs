use serde::{Deserialize, Serialize};

/// A hex id which implements a Display of hexadecimal digits; used in
/// conjunction with the id generation backend to provide the globally
/// unique identifiers that PMR2 had.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HexId {
    id: i64,
}

pub mod traits;
mod impls;
