use leptos::prelude::*;

pub struct ViewAvailableItem {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
}

pub struct ViewsAvailableCtx(pub Vec<ViewAvailableItem>);

#[component]
pub(in crate::app) fn ViewsAvailable() -> impl IntoView {
    let ctx = expect_context::<ArcReadSignal<Option<ViewsAvailableCtx>>>()
        .with(|ctx| ctx.as_ref().map(|_ctx|
            view! {
                <h4>"Views Available"</h4>
            }
        ));

    view! {
        <section>{ctx}</section>
    }
}
