use async_trait::async_trait;
use crate::{
    platform::PlatformUrl,
    ac::{
        traits::{
            PolicyBackend,
            ResourceBackend,
            UserBackend,
        },
    },
};

/// ACPlatform - Access Control Platform
///
/// This platform is used to persist access control information for PMR.
///
/// This trait is applicable to everything that correctly implements the
/// relevant backends that compose this trait.
#[async_trait]
pub trait ACPlatform: PolicyBackend
    + ResourceBackend
    + UserBackend

    + PlatformUrl

    + Send
    + Sync
{
    fn as_dyn(&self) -> &(dyn ACPlatform);
}

impl<P: PolicyBackend
    + ResourceBackend
    + UserBackend

    + PlatformUrl

    + Send
    + Sync
> ACPlatform for P {
    fn as_dyn(&self) -> &(dyn ACPlatform) {
        self
    }
}
