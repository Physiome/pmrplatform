use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Serialize, Deserialize};

use crate::ac::WorkflowState;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContentActionItem {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
}

// Even if the Resource wrapping this is optional, the inner can still be
// None to help with error while processing the resource, and have that be
// distinct from a thing that offers no additional pages if the goal is to
// also keep the portlet visible.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentActionCtx(pub Option<Vec<ContentActionItem>>);

#[component]
pub fn ContentAction() -> impl IntoView {
    use_context::<ReadSignal<Resource<ContentActionCtx>>>().map(|ctx| {
        let resource = ctx.get();
        let action_view = move || Suspend::new(async move {
            resource.await.0.map(|action| {
                let view = action.into_iter()
                    .map(|ContentActionItem { href, text, .. }| view! {
                        <li><A href>{text}</A></li>
                    })
                    .collect_view();
                view! {
                    <nav>
                        <ul>
                            {view}
                        </ul>
                    </nav>
                    <div class="flex-grow"></div>
                }
            })
        });
        view! {
            <section id="content-action">
                <Transition>{action_view}</Transition>
                <WorkflowState/>
            </section>
        }
    })
}
