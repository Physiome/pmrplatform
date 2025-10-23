use async_trait::async_trait;
use crate::{
    citation::traits::CitationBackend,
    // error::BackendError,
    platform::PlatformUrl,
};

/// PCPlatform - Processed Content Platform
///
/// This platform is used to provide storage and access to processed content
/// within PMR.
///
/// Processed Content are content derived from the managed content.  Examples
/// include indexes for search engines, metadata listing, and everything else
/// that require some form of RDBMS to better link across all related managed
/// content that have been stored on the overall platform itself.
#[async_trait]
pub trait PCPlatform: PlatformUrl
    + CitationBackend
    // TODO need to determine how this will apply
    // - Goal is to have a better way of doing generic indexes
    // - This is required to facilitate searching/display of summary
    // - work like a long-term cache? basically we have resource_path (as per
    //   citation link), but that will get parsed down somehow.
    // + ResourceBackend

    + Send
    + Sync
{
    fn as_dyn(&self) -> &dyn PCPlatform;
}

pub trait DefaultPCPlatform: PCPlatform {}

impl<P: PlatformUrl
    + CitationBackend

    + DefaultPCPlatform

    + Send
    + Sync
> PCPlatform for P {
    fn as_dyn(&self) -> &dyn PCPlatform {
        self
    }
}
