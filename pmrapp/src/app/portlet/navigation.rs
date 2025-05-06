use leptos::prelude::*;
use leptos_router::components::A;
use leptos_sync_ssr::portlet::PortletCtx;
use serde::{Serialize, Deserialize};

use crate::error::AppError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationItem {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationItems(Vec<NavigationItem>);

pub type NavigationCtx = PortletCtx<NavigationItems, AppError>;

impl IntoRender for NavigationItems {
    type Output = AnyView;

    fn into_render(self) -> Self::Output {
        let view = self.0
            .into_iter()
            .map(|NavigationItem { href, text, .. }| view! {
                <li><A href>{text}</A></li>
            })
            .collect_view();
        view! {
            <section>
                <h4>"Navigation"</h4>
                <nav>
                    <ul>
                        {view}
                    </ul>
                </nav>
            </section>
        }
        .into_any()
    }
}

impl From<NavigationItems> for Vec<NavigationItem> {
    fn from(value: NavigationItems) -> Self {
        value.0
    }
}

impl From<Vec<NavigationItem>> for NavigationItems {
    fn from(value: Vec<NavigationItem>) -> Self {
        Self(value)
    }
}

#[component]
pub fn Navigation() -> impl IntoView {
    NavigationCtx::render()
}
