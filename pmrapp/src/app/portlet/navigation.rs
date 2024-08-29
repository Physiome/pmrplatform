use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct NavigationItem {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
}

#[derive(Clone, Debug)]
pub struct NavigationCtx(pub Vec<NavigationItem>);

#[component]
pub(in crate::app) fn Navigation() -> impl IntoView {
    let navigation_ctx = expect_context::<ArcReadSignal<Option<NavigationCtx>>>();
    move || navigation_ctx.get().map(|navigation| view! {
        <section>
            <h4>{format!("Navigation {:?}", navigation.0.len())}</h4>
        </section>
    })
}
