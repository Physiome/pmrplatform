#[derive(Copy, Clone, Default, PartialEq)]
pub struct SessionToken(u128);

#[cfg(feature="session")]
mod factory;
#[cfg(feature="session")]
pub use factory::*;

mod impls;
