#[derive(Copy, Clone, PartialEq)]
pub struct SessionToken(u128);

#[cfg(feature="server")]
mod builder;
#[cfg(feature="server")]
pub use builder::*;

mod impls;
