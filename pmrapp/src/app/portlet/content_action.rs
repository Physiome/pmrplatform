use leptos::prelude::*;
use leptos_router::components::A;
use pmrcore::ac::traits::GenpolEnforcer as _;
use pmrrbac::PolicyEnforcer;
use serde::{Serialize, Deserialize};

use crate::{
    ac::{
        AccountCtx,
        WorkflowState,
    },
    enforcement::PolicyState,
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
pub struct ContentActionCtx {
    // This is the current owner of the action context menu
    current_owner: Option<String>,
    value: Option<Vec<ContentActionItem>>,
}

impl ContentActionCtx {
    pub fn new(
        current_owner: String,
        value: Vec<ContentActionItem>,
    ) -> Self {
        Self {
            current_owner: Some(current_owner),
            value: Some(value),
        }
    }

    pub fn clear(&mut self) {
        self.current_owner = None;
        self.value = None;
    }

    pub fn set(&mut self, current_owner: String, value: Vec<ContentActionItem>) {
        self.current_owner = Some(current_owner);
        self.value = Some(value);
    }

    pub fn replace(&mut self, Self { current_owner, value }: Self) {
        self.current_owner = current_owner;
        self.value = value;
    }

    pub fn reset_for(&mut self, current_owner: &str) {
        if self.current_owner.as_deref() == Some(current_owner) {
            leptos::logging::log!("reset for {current_owner}");
            self.clear();
            if let Some(account_ctx) = use_context::<AccountCtx>() {
                leptos::logging::log!("also reset AccountCtx res_policy_state");
                account_ctx.set_ps.set(PolicyState::default());
            }
        }
    }
}

#[component]
pub fn ContentAction() -> impl IntoView {
    let account_ctx = expect_context::<AccountCtx>();
    // TODO the res_policy_state must be integrated with the result from the data that
    // is returned for populating contentaction
    use_context::<ReadSignal<Resource<ContentActionCtx>>>().map(move |ctx| {
        let res_ctx = ctx.get();
        let action_view = move || {
            let res_ps = account_ctx.res_ps.clone();
            Suspend::new(async move {
                let enforcer = PolicyEnforcer::from(
                    res_ps.await.policy
                        .unwrap_or_default()
                );
                res_ctx.await.value.map(|action| {
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
