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
    let ctx = expect_context::<ArcReadSignal<Option<Resource<ViewsAvailableCtx>>>>();
    move || {
        let resource = ctx.get();
        view! {
            <Transition>{
                move || Suspend::new(async move {
                    // TODO maybe set a flag via a signal so that the appropriate class
                    // can be calculated to avoid the sidebar grid space being reserved?
                    // Unless of course there is a CSS-based solution.
                    match resource {
                        Some(resource) => resource.await.0.map(|views_available| view! {
                            <section>
                                <h4>"Views Available: "{views_available.len()}</h4>
                            </section>
                        }),
                        _ => None,
                    }
                })
            }</Transition>
        }
    }
}
