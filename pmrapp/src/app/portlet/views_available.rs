use leptos::prelude::*;
use leptos_router::components::A;
use leptos_sync_ssr::portlet::PortletCtx;
use pmrcore::exposure::ExposureFile;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewsAvailableItem {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewsAvailableItems(Vec<ViewsAvailableItem>);

pub type ViewsAvailableCtx = PortletCtx<ViewsAvailableItems>;

impl IntoRender for ViewsAvailableItems {
    type Output = AnyView;

    fn into_render(self) -> Self::Output {
        let view = self.0
            .into_iter()
            .map(|ViewsAvailableItem { href, text, .. }| view! {
                <li><A href>{text}</A></li>
            })
            .collect_view();
        view! {
            <section>
                <h4>"Views Available"</h4>
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

#[component]
pub fn ViewsAvailable() -> impl IntoView {
    ViewsAvailableCtx::render()
}

impl From<ViewsAvailableItems> for Vec<ViewsAvailableItem> {
    fn from(value: ViewsAvailableItems) -> Self {
        value.0
    }
}

impl From<Vec<ViewsAvailableItem>> for ViewsAvailableItems {
    fn from(value: Vec<ViewsAvailableItem>) -> Self {
        Self(value)
    }
}

impl From<&ExposureFile> for ViewsAvailableItems {
    fn from(item: &ExposureFile) -> Self {
        let exposure_id = item.exposure_id;
        let file = item.workspace_file_path.clone();
        Self(item.views
            .as_ref()
            .map(|views| {
                views
                    .iter()
                    .filter_map(|view| {
                        view.view_key.as_ref().map(|view_key| ViewsAvailableItem {
                            href: format!("/exposure/{exposure_id}/{file}/{view_key}"),
                            // TODO should derive from exposure.files when it contains title/description
                            text: view_key.clone(),
                            title: None,
                        })
                    })
            })
            .unwrap()
            .collect::<Vec<_>>()
            .into()
        )
    }
}
