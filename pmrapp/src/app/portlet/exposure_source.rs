use leptos::prelude::*;
use leptos_sync_ssr::portlet::PortletCtx;
use serde::{Serialize, Deserialize};

use crate::error::AppError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExposureSourceItem {
    pub commit_id: String,
    pub workspace_id: String,
    pub workspace_title: String,
}

pub type ExposureSourceCtx = PortletCtx<ExposureSourceItem, AppError>;

impl IntoRender for ExposureSourceItem {
    type Output = AnyView;

    fn into_render(self) -> Self::Output {
        let Self { commit_id, workspace_id, workspace_title } = self;
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
        .into_any()
    }
}

#[component]
pub fn ExposureSource() -> impl IntoView {
    ExposureSourceCtx::render()
}
