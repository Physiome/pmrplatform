#[derive(Copy, Clone, PartialEq)]
pub struct SessionToken(u128);

#[cfg(feature="server")]
mod factory;
#[cfg(feature="server")]
pub use factory::*;

mod impls;
