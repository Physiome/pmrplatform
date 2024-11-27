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

fn provide_portlet_context_for<
    T: Clone
    + Default
    + Send
    + Sync
    + PartialEq
    + Serialize
    // ideally this bound shouldn't be here, but this makes it work for now...
    + for<'de> Deserialize<'de>
    + 'static
>() {
    // While providing the context behind a resource feels very much
    // superfluous, it does help avoid hydration issues as it functions
    // as a way to somehow inform the reactive system that this needs
    // some awaiting to do, as the underlying data may be provided via
    // a server function.  While just using the read signal does work
    // for CSR, but that results in hydration issue from mismatch with
    // SSR render, thus making the simpler approach unsuitable for use.
    let (rs, ws) = signal(T::default());
    let (ctx, _) = signal(Resource::new(
        move || rs.get(),
        |rs| async move { rs },
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
