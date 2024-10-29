mod builder;
#[cfg(feature = "casbin")]
pub mod casbin;
pub mod error;
pub mod simple;

pub use simple::PolicyEnforcer;
pub use builder::Builder;

pub trait Enforcer: pmrcore::ac::traits::Enforcer<Error=error::Error> {}
impl<T: pmrcore::ac::traits::Enforcer<Error=error::Error>
    + Send
    + Sync
> Enforcer for T {}
