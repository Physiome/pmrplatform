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
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ExposureSourceCtx(Option<ExposureSourceItem>);

impl ExposureSourceCtx {
    pub fn clear(&mut self) {
        self.0 = None;
    }

    pub fn set(&mut self, value: ExposureSourceItem) {
        self.0 = Some(value);
    }

    pub fn replace(&mut self, value: Self) {
        self.0 = value.0;
    }
}

#[component]
pub fn ExposureSource() -> impl IntoView {
    use_context::<ReadSignal<Resource<ExposureSourceCtx>>>().map(|ctx| {
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
                                    <a href=format!("/workspace/{workspace_id}/")>{workspace_title.clone()}</a>
                                    " at changeset "
                                    <a href=format!("/workspace/{workspace_id}/file/{commit_id}/")>
                                        {commit_id.get(..12).unwrap_or(&commit_id).to_string()}
                                    </a>
                                    ".
                                "</div>
                            </section>
                        }
                    })
                })
            }</Transition>
        }
    })
}

impl From<ExposureSourceItem> for ExposureSourceCtx {
    fn from(item: ExposureSourceItem) -> Self {
        Self(Some(item))
    }
}

impl From<Option<ExposureSourceItem>> for ExposureSourceCtx {
    fn from(item: Option<ExposureSourceItem>) -> Self {
        Self(item)
    }
}
