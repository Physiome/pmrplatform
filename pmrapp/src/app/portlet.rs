pub(super) mod content_action;
pub(super) mod exposure_source;
pub(super) mod navigation;
pub(super) mod views_available;

pub use self::content_action::{
    ContentAction,
    ContentActionCtx,
    ContentActionItem,
};
pub use self::exposure_source::{
    ExposureSource,
    ExposureSourceCtx,
    ExposureSourceItem,
};
pub use self::navigation::{
    Navigation,
    NavigationCtx,
    NavigationItem,
};
pub use self::views_available::{
    ViewsAvailable,
    ViewsAvailableCtx,
    ViewsAvailableItem,
};

pub fn provide_portlet_context() {
    ContentActionCtx::provide();
    NavigationCtx::provide();
    ViewsAvailableCtx::provide();
    ExposureSourceCtx::provide();
}
