use async_trait::async_trait;
use crate::{
    platform::{
        PlatformCore,
        RawPlatform,
    },
    ac::{
        traits::{
            PolicyBackend,
            ResourceBackend,
            SessionBackend,
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
    + SessionBackend

    + PlatformCore

    + Send
    + Sync
{
    fn as_dyn(&self) -> &dyn ACPlatform;
}

pub trait DefaultACPlatform: ACPlatform {}
pub trait RawACPlatform: ACPlatform + RawPlatform {}

impl<P: PolicyBackend
    + ResourceBackend
    + UserBackend
    + SessionBackend

    + PlatformCore

    + DefaultACPlatform

    + Send
    + Sync
> ACPlatform for P {
    fn as_dyn(&self) -> &dyn ACPlatform {
        self
    }
}

impl <P: ACPlatform + RawPlatform> RawACPlatform for P {}
