use leptos::prelude::*;

pub struct NavigationItem {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
}

pub struct NavigationCtx(pub Vec<NavigationItem>);

#[component]
pub(in crate::app) fn Navigation() -> impl IntoView {
    let ctx = expect_context::<ArcReadSignal<Option<NavigationCtx>>>()
        .with(|ctx| ctx.as_ref().map(|_ctx|
            view! {
                <h4>"Navigation"</h4>
            }
        ));

    view! {
        <section>{ctx}</section>
    }
}
