use leptos::prelude::*;
use serde::{Serialize, Deserialize};

pub(super) mod content_action;
pub(super) mod exposure_source;
pub(super) mod navigation;
pub(super) mod views_available;

pub use self::content_action::{
    ContentAction,
    ContentActionItem,
    ContentActionCtx,
};
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

fn provide_portlet_context_for<T: Clone + Default + Send + Sync + PartialEq + Serialize + for<'de> Deserialize<'de> + 'static>() {
    let (rs, ws) = signal(None::<T>);
    let (ctx, _) = signal(Resource::new(
        move || rs.get(),
        |rs| async move { rs.unwrap_or(T::default()) },
    ));
    provide_context(ctx);
    provide_context(ws);
}

pub fn provide_portlet_context() {
    provide_portlet_context_for::<ContentActionCtx>();
    provide_portlet_context_for::<ExposureSourceCtx>();
    provide_portlet_context_for::<NavigationCtx>();
    provide_portlet_context_for::<ViewsAvailableCtx>();
}
