use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct ViewAvailableItem {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ViewsAvailableCtx(pub Option<Vec<ViewAvailableItem>>);

#[component]
pub(in crate::app) fn ViewsAvailable() -> impl IntoView {
    let ctx = expect_context::<ArcReadSignal<ViewsAvailableCtx>>();
    move || ctx.get().0.map(|views_available| view! {
        <section>
            <h4>"Views Available "{views_available.len()}</h4>
        </section>
    })
}
