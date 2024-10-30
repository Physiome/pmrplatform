use leptos::prelude::*;
use pmrcore::ac::{
    genpolicy::Policy,
    workflow::State,
};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContentActionItem {
    // FIXME AccountCtx does encapsulate this whole item as a resource already,
    // ideally this shouldn't need the duplication here, but keeping it like so
    // for consistency.
    pub policy: Policy,
    pub workflow_state: State,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentActionCtx(pub Option<ContentActionItem>);

#[component]
pub fn ContentAction() -> impl IntoView {
    use_context::<ReadSignal<Resource<ContentActionCtx>>>().map(|ctx| {
        let resource = ctx.get();
        view! {
            <Transition>{
                move || Suspend::new(async move {
                    resource.await.0.map(|ContentActionItem { policy, workflow_state }| {
                        view! {
                            <section>
                                <a href=policy.resource.to_owned()>"Resource Top"</a>
                                <div>{workflow_state.to_string()}</div>
                            </section>
                        }
                    })
                })
            }</Transition>
        }
    })
}

impl From<ContentActionItem> for ContentActionCtx {
    fn from(item: ContentActionItem) -> Self {
        Self(Some(item))
    }
}
