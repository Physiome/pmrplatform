use leptos::prelude::*;
use leptos_router::components::A;
use pmrcore::ac::{
    agent::Agent,
    traits::GenpolEnforcer as _,
};
use pmrrbac::PolicyEnforcer;
use serde::{Serialize, Deserialize};

use crate::ac::{
    AccountCtx,
    WorkflowState,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContentActionItem {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
    pub req_action: Option<String>,
}

// Even if the Resource wrapping this is optional, the inner can still be
// None to help with error while processing the resource, and have that be
// distinct from a thing that offers no additional pages if the goal is to
// also keep the portlet visible.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentActionCtx(pub Option<Vec<ContentActionItem>>);

#[component]
pub fn ContentAction() -> impl IntoView {
    let account_ctx = expect_context::<AccountCtx>();
    use_context::<ReadSignal<Resource<ContentActionCtx>>>().map(|ctx| {
        let res_ctx = ctx.get();
        let action_view = move || {
            let current_user = account_ctx.current_user.clone();
            let res_policy_state = account_ctx.res_policy_state.clone();
            Suspend::new(async move {
                let agent = current_user.await
                    .ok()
                    .flatten()
                    .map(Agent::from)
                    .unwrap_or_default();
                let enforcer = PolicyEnforcer::from({
                    let policy = res_policy_state.await
                        .ok()
                        .flatten()
                        .map(|(policy, _)| policy)
                        .unwrap_or_default();
                    leptos::logging::log!("{:?}", &policy);
                    policy
                });
                res_ctx.await.0.map(|action| {
                    let view = action.into_iter()
                        .filter_map(|ContentActionItem { href, text, title, req_action }| {
                            req_action.as_ref()
                                .map(|action| enforcer
                                    .enforce(action)
                                    .unwrap_or(false))
                                .unwrap_or(true)
                                .then(|| view! {
                                    <li><A href attr:title=title>{text}</A></li>
                                })
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
            })
        };
        view! {
            <section id="content-action">
                <Transition>{action_view}</Transition>
                <WorkflowState/>
            </section>
        }
    })
}
