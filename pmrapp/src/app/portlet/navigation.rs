use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct NavigationItem {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
}

#[derive(Clone, Debug)]
pub struct NavigationCtx(pub Option<Vec<NavigationItem>>);

#[component]
pub(in crate::app) fn Navigation() -> impl IntoView {
    // TODO this might actually need to be a Resource<NavigationCtx>, and
    // the set context will have to be done at the root of the element so it
    // picks up in time?
    let ctx = expect_context::<ArcReadSignal<NavigationCtx>>();
    move || {
        ctx.get().0.map(|navigation| {
            view! {
                <section>
                    <h4>"Navigation: "{navigation.len()}</h4>
                </section>
            }
        })
    }
}
