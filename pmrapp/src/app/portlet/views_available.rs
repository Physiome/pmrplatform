use leptos::prelude::*;
use leptos_router::components::A;
use pmrcore::exposure::ExposureFile;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewsAvailableItem {
    pub href: String,
    pub text: String,
    pub title: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewsAvailableCtx(pub Option<Vec<ViewsAvailableItem>>);

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
                        Some(resource) => resource.await.0.map(|views_available| {
                            let view = views_available.into_iter()
                                .map(|ViewsAvailableItem { href, text, .. }| view! {
                                    <li><A href>{text}</A></li>
                                })
                                .collect_view();
                            view! {
                                <section>
                                    <h4>"Views Available"</h4>
                                    <ul>
                                        {view}
                                    </ul>
                                </section>
                            }
                        }),
                        _ => None,
                    }
                })
            }</Transition>
        }
    }
}

impl From<&ExposureFile> for ViewsAvailableCtx {
    fn from(item: &ExposureFile) -> Self {
        let exposure_id = item.exposure_id;
        let file = item.workspace_file_path.clone();
        Self(item.views.as_ref().map(|views| views.iter()
            .filter_map(|view| {
                view.view_key.as_ref().map(|view_key| ViewsAvailableItem {
                    href: format!("/exposure/{exposure_id}/{file}/{view_key}"),
                    // TODO should derive from exposure.files when it contains title/description
                    text: view_key.clone(),
                    title: None,
                })
            })
            .collect::<Vec<_>>()
        ).into())
    }
}
