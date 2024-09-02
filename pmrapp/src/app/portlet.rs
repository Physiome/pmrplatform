pub(super) mod exposure_source;
pub(super) mod navigation;
pub(super) mod views_available;

pub use self::exposure_source::{
    ExposureSourceItem,
    ExposureSourceCtx,
};
pub use self::navigation::{
    NavigationItem,
    NavigationCtx,
};
pub use self::views_available::{
    ViewsAvailableItem,
    ViewsAvailableCtx,
};

pub fn provide_portlet_context() {
    self::exposure_source::provide_exposure_source_portlet_context();
    self::navigation::provide_navigation_portlet_context();
    self::views_available::provide_views_available_portlet_context();
}
