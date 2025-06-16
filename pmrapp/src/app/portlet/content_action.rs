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
    pub parent: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ContentAction {
    action: Vec<ContentActionItem>,
}

pub type ContentActionCtx = PortletCtx<ContentAction>;

#[component]
pub fn ContentAction() -> impl IntoView {
    view! {
        <section id="content-action">
            {ContentActionCtx::render()}
            <WorkflowState/>
        </section>
    }
}

impl IntoRender for ContentAction {
    type Output = AnyView;

    fn into_render(self) -> Self::Output {
        let account_ctx = expect_context::<AccountCtx>();
        // let res_ps = account_ctx.res_ps.clone();
        // let enforcer = PolicyEnforcer::from(
        //     res_ps.await.policy
        //         .unwrap_or_default()
        // );
        // let view = action.into_iter()
        //     .filter_map(|ContentActionItem { href, text, title, req_action }| {
        //         req_action.as_ref()
        //             .map(|action| enforcer
        //                 .enforce(action)
        //                 .unwrap_or(false))
        //             .unwrap_or(true)
        //             .then(|| view! {
        //                 <li><A href attr:title=title>{text}</A></li>
        //             })
        //     })
        //     .collect_view();
        view! {
            <nav>
                <ul>
                    // {view}
                </ul>
            </nav>
            <div class="flex-grow"></div>
        }
        .into_any()
    }
}
