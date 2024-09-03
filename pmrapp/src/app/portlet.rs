pub(super) mod exposure_source;
pub(super) mod navigation;
pub(super) mod views_available;

pub use self::exposure_source::{
    ExposureSource,
    ExposureSourceItem,
    ExposureSourceCtx,
};
pub use self::navigation::{
    Navigation,
    NavigationItem,
    NavigationCtx,
};
pub use self::views_available::{
    ViewsAvailable,
    ViewsAvailableItem,
    ViewsAvailableCtx,
};

pub fn provide_portlet_context() {
    self::exposure_source::provide_exposure_source_portlet_context();
    self::navigation::provide_navigation_portlet_context();
    self::views_available::provide_views_available_portlet_context();
}
