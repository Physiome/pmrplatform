use leptos::prelude::*;
use leptos_router::components::A;
use leptos_sync_ssr::portlet::PortletCtx;
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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentActions {
    actions: Vec<(String, ContentActionItem)>,
}

impl ContentActions {
    pub fn new(parent: &str, items: Option<Vec<ContentActionItem>>) -> Self {
        let actions = items.unwrap_or_default()
            .into_iter()
            .map(|item| (parent.to_string(), item))
            .collect();
        Self { actions }
    }

    pub fn update(&mut self, parent: &str, items: Option<Vec<ContentActionItem>>) {
        self.actions.retain(|(p, _)| (parent != p));
        if let Some(items) = items {
            let mut new = items.into_iter()
                .map(|item| (parent.to_string(), item))
                .collect();
            self.actions.append(&mut new);
        }
    }
}

impl From<(&str, Option<Vec<ContentActionItem>>)> for ContentActions {
    fn from((parent, items): (&str, Option<Vec<ContentActionItem>>)) -> Self {
        Self::new(parent, items)
    }
}

pub type ContentActionCtx = PortletCtx<ContentActions>;

#[component]
pub fn ContentAction() -> impl IntoView {
    view! {
        <section id="content-action">
            {ContentActionCtx::render()}
            // this div was originally inside the above, but having that there
            // messes up hydration such the above is reordered to be the bottom.
            <div class="flex-grow"></div>
            <WorkflowState/>
        </section>
    }
}

impl IntoRender for ContentActions {
    type Output = AnyView;

    fn into_render(self) -> Self::Output {
        let account_ctx = expect_context::<AccountCtx>();
        let view = move || {
            let res_ps = account_ctx.policy_state.read_only();
            let actions = self.actions.clone();
            Suspend::new(async move {
                let policy = res_ps.await
                    .map(|ps| ps.policy)
                    .unwrap_or_default()
                    .unwrap_or_default();
                let enforcer = PolicyEnforcer::from(policy);
                actions.into_iter()
                    .filter_map(|(_, ContentActionItem { href, text, title, req_action })| {
                        req_action.as_ref()
                            .map(|action| enforcer
                                .enforce(action)
                                .unwrap_or(false))
                            .unwrap_or(true)
                            .then(|| view! {
                                <li><A href attr:title=title>{text}</A></li>
                            })
                    })
                    .collect_view()
            })
        };

        view! {
            <nav>
                <ul>
                    <Suspense>{view}</Suspense>
                </ul>
            </nav>
            // <div class="flex-grow"></div>
        }
        .into_any()
    }
}
