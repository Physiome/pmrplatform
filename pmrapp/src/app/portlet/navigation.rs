use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationItem {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
}

// Even if the Resource wrapping this is optional, the inner can still be
// None to help with error while processing the resource, and have that be
// distinct from a thing that offers no additional pages if the goal is to
// also keep the portlet visible.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationCtx(pub Option<Vec<NavigationItem>>);

#[component]
pub fn Navigation() -> impl IntoView {
    // TODO this might actually need to be a Resource<NavigationCtx>, and
    // the set context will have to be done at the root of the element so it
    // picks up in time?
    let ctx = expect_context::<ArcReadSignal<Resource<NavigationCtx>>>();
    let resource = ctx.get();
    view! {
        <Transition>{
            move || Suspend::new(async move {
                // TODO maybe set a flag via a signal so that the appropriate class
                // can be calculated to avoid the sidebar grid space being reserved?
                // Unless of course there is a CSS-based solution.
                resource.await.0.map(|navigation| {
                    let view = navigation.into_iter()
                        .map(|NavigationItem { href, text, .. }| view! {
                            <li><A href>{text}</A></li>
                        })
                        .collect_view();
                    view! {
                        <section>
                            <h4>"Navigation"</h4>
                            <ul>
                                {view}
                            </ul>
                        </section>
                    }
                })
            })
        }</Transition>
    }
}

pub(super) fn provide_navigation_portlet_context() {
    let (navigation, set_navigation) = signal(None::<NavigationCtx>);
    let (navigation_ctx, _) = arc_signal(Resource::new(
        move || navigation.get(),
        |navigation| async move { navigation.unwrap_or(NavigationCtx(None)) },
    ));
    provide_context(navigation_ctx);
    provide_context(set_navigation);
}
