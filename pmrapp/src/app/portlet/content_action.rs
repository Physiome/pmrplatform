use leptos::prelude::*;
use leptos_router::components::A;
use leptos_sync_ssr::portlet::PortletCtx;
use pmrcore::ac::traits::GenpolEnforcer as _;
use pmrrbac::PolicyEnforcer;
use serde::{Serialize, Deserialize};

use crate::ac::{AccountCtx, WorkflowState};

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
    pub fn new<S>(parent: S, items: Option<Vec<ContentActionItem>>) -> Self
    where
        S: AsRef<str> + ToString,
    {
        let actions = items.unwrap_or_default()
            .into_iter()
            .map(|item| (parent.to_string(), item))
            .collect();
        Self { actions }
    }

    pub fn update<S>(&mut self, parent: S, items: Option<Vec<ContentActionItem>>)
    where
        S: AsRef<str> + ToString,
    {
        self.actions.retain(|(p, _)| parent.as_ref() != p);
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
pub fn ContentActionItems() -> impl IntoView {
    let account_ctx = expect_context::<AccountCtx>();
    let ctx = expect_context::<ContentActionCtx>();
    let resource = ctx.inner_resource();

    let suspend = move || {
	let resource = resource.clone();
        // FIXME unsure why this resource needs manual tracking here.
        // this is due to some views not actually having this show.
        #[cfg(not(feature = "ssr"))]
        resource.track();
        let res_ps = account_ctx.policy_state.read_only();
	Suspend::new(async move {
            leptos::logging::log!("ContentActionItems Suspend");
	    let actions = resource.await?.actions;
            leptos::logging::log!("{actions:?}");
            let policy = res_ps.await
                .map(|ps| ps.policy)
                .unwrap_or_default()
                .unwrap_or_default();
            let enforcer = PolicyEnforcer::from(policy);
            let view = actions.into_iter()
                .filter_map(|(_, ContentActionItem { href, text, title, req_action })| {
                    req_action.as_ref()
                        .map(|action| enforcer
                            .enforce(action)
                            .unwrap_or(false))
                        .unwrap_or(true)
                        .then(|| view! {
                            <li><A href exact=true attr:title=title>{text}</A></li>
                        })
                })
                .collect_view();
            Some(view! {
                <nav>
                    <ul>
                        {view}
                    </ul>
                </nav>
                <div class="flex-grow"></div>
            })
	})
    };
    view! { <Transition>{suspend}</Transition> }
}

#[component]
pub fn ContentAction() -> impl IntoView {
    view! {
        <section id="content-action">
            <ContentActionItems/>
            <WorkflowState/>
        </section>
    }
}
