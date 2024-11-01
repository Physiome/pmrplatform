use serde::{Deserialize, Serialize};

mod impls;
pub mod transition;
pub use transition::Transition;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum State {
    // catch-all when infallable conversion is required
    #[default]
    Unknown,
    Private,
    Pending,
    Published,
    Expired,
}
