use leptos::prelude::*;
use leptos_sync_ssr::portlet::PortletCtx;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExposureSourceItem {
    pub commit_id: String,
    pub workspace_alias: Option<String>,
    pub workspace_id: String,
    pub workspace_title: String,
}

pub type ExposureSourceCtx = PortletCtx<ExposureSourceItem>;

impl IntoRender for ExposureSourceItem {
    type Output = AnyView;

    fn into_render(self) -> Self::Output {
        let Self { commit_id, workspace_alias, workspace_id, workspace_title } = self;
        let prefix = workspace_alias.map_or_else(
            || format!("/workspace/:/id/{workspace_id}/"),
            |alias| format!("/workspace/{alias}/"),
        );
        view! {
            <section>
                <h4>"Source"</h4>
                <div>"
                    Derived from workspace "
                    <a href=prefix.clone()>{workspace_title.clone()}</a>
                    " at changeset "
                    <a href=format!("{prefix}file/{commit_id}/")>
                        {commit_id.get(..12).unwrap_or(&commit_id).to_string()}
                    </a>
                    ".
                "</div>
            </section>
        }
        .into_any()
    }
}

#[component]
pub fn ExposureSource() -> impl IntoView {
    ExposureSourceCtx::render()
}
