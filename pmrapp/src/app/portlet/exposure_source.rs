use leptos::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExposureSourceItem {
    pub commit_id: String,
    pub workspace_id: String,
    pub workspace_title: String,
}

// Even if the Resource wrapping this is optional, the inner can still be
// None to help with error while processing the resource, and have that be
// distinct from a thing that offers no additional pages if the goal is to
// also keep the portlet visible.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExposureSourceCtx(pub Option<ExposureSourceItem>);

#[component]
pub fn ExposureSource() -> impl IntoView {
    let ctx = expect_context::<ArcReadSignal<Resource<ExposureSourceCtx>>>();
    let resource = ctx.get();
    view! {
        <Transition>{
            move || Suspend::new(async move {
                // TODO maybe set a flag via a signal so that the appropriate class
                // can be calculated to avoid the sidebar grid space being reserved?
                // Unless of course there is a CSS-based solution.
                resource.await.0.map(|ExposureSourceItem { commit_id, workspace_id, workspace_title }| {
                    view! {
                        <section>
                            <h4>"Source"</h4>
                            <div>"
                                Derived from workspace "
                                <a href=format!("/workspace/{workspace_id}")>{workspace_title.clone()}</a>
                                " at changeset "
                                <a href=format!("/workspace/{workspace_id}/file/{commit_id}/")>
                                    {commit_id[..12].to_string()}
                                </a>
                                ".
                            "</div>
                        </section>
                    }
                })
            })
        }</Transition>
    }
}

pub(super) fn provide_exposure_source_portlet_context() {
    let (exposure_source, set_exposure_source) = signal(None::<ExposureSourceCtx>);
    let (exposure_source_ctx, _) = arc_signal(Resource::new(
        move || exposure_source.get(),
        |exposure_source| async move { exposure_source.unwrap_or(ExposureSourceCtx(None)) },
    ));
    provide_context(exposure_source_ctx);
    provide_context(set_exposure_source);
}

impl From<ExposureSourceItem> for ExposureSourceCtx {
    fn from(item: ExposureSourceItem) -> Self {
        Self(Some(item))
    }
}
