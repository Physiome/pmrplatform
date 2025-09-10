use enumset::EnumSetType;
use serde::{Deserialize, Serialize};

mod impls;
pub mod transition;
pub use transition::Transition;

#[non_exhaustive]
#[derive(Debug, Default, EnumSetType, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    // catch-all when infallable conversion is required
    #[default]
    Unknown,
    Private,
    Pending,
    Published,
    Expired,
}
